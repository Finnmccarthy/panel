import { CollisionDetection, closestCenter, DndContext, DragOverlay, Modifier } from '@dnd-kit/core';
import { SortableContext, SortingStrategy, useSortable, verticalListSortingStrategy } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { ComponentProps, CSSProperties, ReactNode, useMemo } from 'react';
import { createPortal } from 'react-dom';
import { createDropAnimation, useDndSensors, useDndState } from '@/lib/dragAndDrop.ts';
import { useTranslations } from '@/providers/TranslationProvider.tsx';

export type DndItem = {
  id: string;
};

export interface DndConfig {
  pointerActivationDistance?: number;
  touchActivationDelay?: number;
  touchActivationTolerance?: number;
  dragOverlayDuration?: number;
  dragOverlayEasing?: string;
}

export interface DndCallbacks<T extends DndItem> {
  onDragStart?: (item: T) => void;
  onDragOver?: (activeId: string, overId: string | null) => void;
  onDragEnd: (items: T[], oldIndex: number, newIndex: number) => void | Promise<void>;
  onDragCancel?: () => void;
  onError?: (error: unknown, originalItems: T[]) => void;
}

export interface SortableItemProps {
  id: string;
  children?: ReactNode;
  disabled?: boolean;
  transitionDuration?: number;
  transitionEasing?: string;
  renderItem?: (props: { isDragging: boolean; dragHandleProps: ComponentProps<'div'> }) => ReactNode;
}

export function SortableItem({
  id,
  children,
  disabled = false,
  transitionDuration = 300,
  transitionEasing = 'cubic-bezier(0.25, 1, 0.5, 1)',
  renderItem,
}: SortableItemProps) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id,
    disabled,
    transition: {
      duration: transitionDuration,
      easing: transitionEasing,
    },
  });

  const style: CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
    touchAction: isDragging ? 'none' : 'manipulation',
  };

  const dragHandleProps = useMemo(
    () => ({
      ...attributes,
      ...listeners,
      style: {
        cursor: isDragging ? 'grabbing' : 'grab',
        touchAction: isDragging ? 'none' : 'manipulation',
      } satisfies CSSProperties,
    }),
    [attributes, listeners, isDragging],
  );

  return (
    <div ref={setNodeRef} style={style} className='min-w-0'>
      {renderItem ? (
        renderItem({ isDragging, dragHandleProps })
      ) : (
        <div {...dragHandleProps} className='h-full min-w-0'>
          {children}
        </div>
      )}
    </div>
  );
}

export interface DndContainerProps<T extends DndItem> {
  items: T[];
  callbacks: DndCallbacks<T>;
  config?: DndConfig;
  strategy?: SortingStrategy;
  collisionDetection?: CollisionDetection;
  children: (items: T[]) => ReactNode;
  renderOverlay?: (activeItem: T | null) => ReactNode;
  modifiers?: Modifier[];
  getItemLabel?: (item: T) => string;
}

const defaultConfig: DndConfig = {};

export function DndContainer<T extends DndItem>({
  items,
  callbacks,
  config = defaultConfig,
  strategy = verticalListSortingStrategy,
  collisionDetection = closestCenter,
  children,
  renderOverlay,
  modifiers,
  getItemLabel,
}: DndContainerProps<T>) {
  const { t } = useTranslations();

  const sensors = useDndSensors(config);
  const dropAnimation = useMemo(() => createDropAnimation(config), [config]);

  const { activeItem, localItems, handleDragStart, handleDragOver, handleDragEnd, handleDragCancel } = useDndState(
    items,
    callbacks,
  );

  const itemIds = useMemo(() => localItems.map((item) => item.id), [localItems]);

  const accessibility = useMemo(() => {
    const describe = (id: string | number) => {
      const index = localItems.findIndex((item) => item.id === id);
      if (index === -1) return String(id);

      const label = getItemLabel?.(localItems[index]);
      return label
        ? t('elements.dragAndDrop.item.labelled', { label, position: index + 1 })
        : t('elements.dragAndDrop.item.unlabelled', { position: index + 1 });
    };

    return {
      announcements: {
        onDragStart: ({ active }) => t('elements.dragAndDrop.announcement.pickedUp', { item: describe(active.id) }),
        onDragOver: ({ active, over }) =>
          over
            ? t('elements.dragAndDrop.announcement.movedOver', {
                item: describe(active.id),
                target: describe(over.id),
              })
            : t('elements.dragAndDrop.announcement.leftTarget', { item: describe(active.id) }),
        onDragEnd: ({ active, over }) =>
          over
            ? t('elements.dragAndDrop.announcement.droppedOn', {
                item: describe(active.id),
                target: describe(over.id),
              })
            : t('elements.dragAndDrop.announcement.dropped', { item: describe(active.id) }),
        onDragCancel: ({ active }) => t('elements.dragAndDrop.announcement.cancelled', { item: describe(active.id) }),
      },
    } satisfies ComponentProps<typeof DndContext>['accessibility'];
  }, [localItems, getItemLabel, t]);

  return (
    <DndContext
      sensors={sensors}
      modifiers={modifiers}
      accessibility={accessibility}
      collisionDetection={collisionDetection}
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDragEnd={handleDragEnd}
      onDragCancel={handleDragCancel}
    >
      <SortableContext items={itemIds} strategy={strategy}>
        {children(localItems)}
      </SortableContext>
      {createPortal(
        <DragOverlay dropAnimation={dropAnimation}>
          {renderOverlay && activeItem ? renderOverlay(activeItem) : null}
        </DragOverlay>,
        document.body,
      )}
    </DndContext>
  );
}
