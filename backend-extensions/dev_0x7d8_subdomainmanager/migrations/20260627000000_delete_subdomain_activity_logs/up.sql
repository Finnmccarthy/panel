DELETE FROM server_activities
WHERE server_activities.event IN ('server:subdomain.create', 'server:subdomain.update', 'server:subdomain.delete');
