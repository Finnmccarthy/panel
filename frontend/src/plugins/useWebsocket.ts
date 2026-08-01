import { useEffect, useRef, useState } from 'react';
import { z } from 'zod';
import { safeParseFromApi } from '@/lib/api-transform.ts';

/**
 * Shared client for the panel's own websocket routes - same-origin paths authenticated by the
 * session cookie. The per-server console socket is a different mechanism (msgpack over an
 * in-band JWT handshake, straight to wings) and lives in `plugins/Websocket.ts`.
 *
 * Passing a `schema` declares that frames are JSON: they are parsed, run through
 * `safeParseFromApi` for the snake_case->camelCase remap plus validation, and handed to
 * `onMessage` typed. Omitting it yields raw string frames. The framing cannot be folded into
 * the schema itself - `applyTransform` unwraps a pipe to its input side, so a
 * `z.string().transform(JSON.parse).pipe(...)` would silently skip the key remap.
 */
interface WebsocketOptions {
  path: string;
  params?: Record<string, string>;
  enabled?: boolean;
  /** milliseconds between reconnect attempts, or null to not reconnect */
  reconnectDelay?: number | null;
  onMessage: (data: string) => void;
  onOpen?: () => void;
  onClose?: (event: CloseEvent) => void;
  /** fires once per loss streak, resetting only once a frame arrives again */
  onConnectionLost?: () => void;
}

type SchemaOptions<T extends z.ZodTypeAny> = Omit<WebsocketOptions, 'onMessage'> & {
  schema: T;
  onMessage: (data: z.infer<T>) => void;
};

export function useWebsocket(options: WebsocketOptions & { schema?: undefined }): { connected: boolean };
export function useWebsocket<T extends z.ZodTypeAny>(options: SchemaOptions<T>): { connected: boolean };

export function useWebsocket<T extends z.ZodTypeAny>(
  options: (WebsocketOptions & { schema?: undefined }) | SchemaOptions<T>,
): { connected: boolean } {
  const { path, params, enabled = true, reconnectDelay = null, schema } = options;

  const [connected, setConnected] = useState(false);

  const handlers = useRef(options);
  handlers.current = options;

  const serializedParams = params ? new URLSearchParams(params).toString() : '';

  useEffect(() => {
    if (!enabled) {
      return;
    }

    let socketRef: WebSocket | null = null;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let destroyed = false;
    let lossNotified = false;
    let lastSchemaError: string | null = null;

    const connect = () => {
      if (destroyed) {
        return;
      }

      const url = new URL(path, window.location.origin);
      url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
      url.search = serializedParams;

      const socket = new WebSocket(url);
      socketRef = socket;

      socket.onopen = () => {
        if (destroyed || socket !== socketRef) {
          return;
        }

        setConnected(true);
        handlers.current.onOpen?.();
      };

      socket.onmessage = (event) => {
        if (destroyed || socket !== socketRef || typeof event.data !== 'string') {
          return;
        }

        if (!schema) {
          lossNotified = false;
          (handlers.current as WebsocketOptions).onMessage(event.data);
          return;
        }

        let raw: unknown;
        try {
          raw = JSON.parse(event.data);
        } catch {
          return;
        }

        const result = safeParseFromApi(schema, raw);
        if (!result.success) {
          // A drifted schema fails every frame, so only report when the failure itself changes
          if (result.message !== lastSchemaError) {
            lastSchemaError = result.message;
            console.error(result.message, '\nfull frame:', raw);
          }
          return;
        }

        lastSchemaError = null;
        lossNotified = false;
        (handlers.current as SchemaOptions<T>).onMessage(result.data);
      };

      socket.onclose = (event) => {
        if (destroyed || socket !== socketRef) {
          return;
        }

        socketRef = null;
        setConnected(false);
        handlers.current.onClose?.(event);

        if (event.wasClean) {
          return;
        }

        if (!lossNotified) {
          lossNotified = true;
          handlers.current.onConnectionLost?.();
        }

        if (reconnectDelay !== null) {
          reconnectTimer = setTimeout(() => {
            reconnectTimer = null;
            connect();
          }, reconnectDelay);
        }
      };
    };

    connect();

    return () => {
      destroyed = true;
      setConnected(false);

      if (reconnectTimer !== null) {
        clearTimeout(reconnectTimer);
      }

      socketRef?.close();
      socketRef = null;
    };
  }, [path, serializedParams, enabled, reconnectDelay, schema]);

  return { connected };
}
