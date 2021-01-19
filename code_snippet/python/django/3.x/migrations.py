# DO NOT do this in migration file, it will cause unexpected recursive call stack error
from django.db import connection

# get apps state at which the latest migration of a application was committed.
def get_app_state_on_migration(self, app_label):
    mi_loader = migrations.loader.MigrationLoader(connection, load=False)
    mi_loader.build_graph()
    # v.applied is datetime object
    mi_key = {k : v.applied for k,v in mi_loader.applied_migrations.items() if k[0] == app_label}
    # get tuple key of the latest migration
    mi_key = max(mi_key)
    pre_commit_state = mi_loader.project_state(mi_key)
    apps = pre_commit_state.apps
    return apps


