# DO NOT do this in migration file, it will cause unexpected recursive call stack error
import logging

from django.conf  import  settings as django_settings
from django.db    import  migrations, router, models, connection
from django.db.utils import OperationalError, ProgrammingError

_logger = logging.getLogger(__name__)


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


class ExtendedRunPython(migrations.RunPython):
    """
    extended from original RunPython class by :
        * providing user-defined arguments to forward function and reverse function
    """
    def __init__(self, code, reverse_code=None, atomic=None, hints=None, elidable=False,
            code_kwargs=None, reverse_code_kwargs=None):
        if code_kwargs:
            self._code_kwargs = code_kwargs
        if reverse_code_kwargs:
            self._reverse_code_kwargs = reverse_code_kwargs
        super().__init__(code=code, reverse_code=reverse_code, atomic=atomic,
                hints=hints, elidable=elidable)

    def deconstruct(self):
        kwargs = {
            'code': self.code,
        }
        if self.reverse_code is not None:
            kwargs['reverse_code'] = self.reverse_code
        if self.atomic is not None:
            kwargs['atomic'] = self.atomic
        if self.hints:
            kwargs['hints'] = self.hints
        if hasattr(self, '_code_kwargs'):
            kwargs['code_kwargs'] = getattr(self, '_code_kwargs')
        if hasattr(self, '_reverse_code_kwargs'):
            kwargs['reverse_code_kwargs'] = getattr(self, '_reverse_code_kwargs')
        return (self.__class__.__qualname__ , [], kwargs)

    def database_forwards(self, app_label, schema_editor, from_state, to_state):
        from_state.clear_delayed_apps_cache()
        if router.allow_migrate(schema_editor.connection.alias, app_label, **self.hints):
            kwargs = getattr(self, '_code_kwargs', {})
            kwargs['app_label'] = app_label
            kwargs['to_state_apps'] = to_state.apps
            self.code(from_state.apps, schema_editor, **kwargs)

    def database_backwards(self, app_label, schema_editor, from_state, to_state):
        if self.reverse_code is None:
            raise NotImplementedError("You cannot reverse this operation")
        if router.allow_migrate(schema_editor.connection.alias, app_label, **self.hints):
            kwargs = getattr(self, '_reverse_code_kwargs', {})
            kwargs['app_label'] = app_label
            kwargs['to_state_apps'] = to_state.apps
            self.reverse_code(from_state.apps, schema_editor, **kwargs)




class AlterTablePrivilege(ExtendedRunPython):
    """
    When there are new models to create, or existing models to rename or delete,
    this operation class is  responsible for updating necessary privileges for
    given database user.
    Note that each instance of this class is supposed to work with ONLY one
    database setup, for the complex migration that includes multi-database updates,
    there should be multiple instances of ths class created & dedicated to that migration.
    """
    PRIVILEGE_MAP = {
        # to create tuple with only one item, you must add comma after the item
        # , otherwise python will NOT recognize the variable as a tuple, instead python
        # treat the variable as the data type of the only item .
        'READ_ONLY' : ('SELECT',),
        'EDIT_ONLY' : ('UPDATE',),
        'WRITE_ONLY': ('INSERT','DELETE','UPDATE'),
        'READ_WRITE': ('SELECT', 'INSERT','DELETE','UPDATE'),
    }

    ACCEPTED_OPERATIONS = (
        migrations.CreateModel, migrations.DeleteModel, migrations.AlterModelTable,
        migrations.AddField,    migrations.RemoveField,
    )

    def __init__(self, autogen_ops, db_setup_tag, **kwargs):
        self._db_setup_tag = db_setup_tag
        self._extract_table_names(autogen_ops)
        code_kwargs = {'operation': self}
        super().__init__(code=_forward_privilege_setup, reverse_code=_backward_privilege_setup,
                code_kwargs=code_kwargs, reverse_code_kwargs=code_kwargs )
        # TODO, figure out how the order of operations affects state change
        # always insert this operation into both ends of the operation list, in case current migration
        # includes complex operations, e.g. create, delete, rename table operations are in
        # a single migration .
        autogen_ops.insert(0, self)
        autogen_ops.append(self)
        self._first_run = True


    def _extract_table_names(self, autogen_ops):
        add_models = []
        rm_models = []
        rename_tables = []

        for op in autogen_ops:
            _priv_lvl = getattr(op, '_priv_lvl', None)
            if _priv_lvl is None:
                continue # discard
            _priv_lvl = ','.join(_priv_lvl)
            if isinstance(op, migrations.CreateModel):
                item = {'model_name': op.name, 'new_table_name': op.options.get('db_table',None), 'priv_lvl': _priv_lvl}
                add_models.append(item)
                for fd in op.fields:
                    if isinstance(fd[1], models.ManyToManyField):
                        item = {'model_name': op.name, 'new_table_name': fd[1].db_table, 'm2m_fd': fd[0],
                        'model_to_table_name_required': True, 'priv_lvl': _priv_lvl}
                        add_models.append(item)
            elif isinstance(op, migrations.AddField) and isinstance(op.field, models.ManyToManyField):
                item = {'model_name': op.model_name, 'new_table_name': op.field.db_table, 'm2m_fd': op.name,
                        'model_to_table_name_required': True, 'priv_lvl': _priv_lvl}
                add_models.append(item)
            elif isinstance(op, migrations.RemoveField):
                # it is not clear whether this field is m2m field, so still add it to remove list
                # and then check at later time when database forward / backward function is running
                item = {'model_name': op.model_name, 'fd': op.name, 'priv_lvl': _priv_lvl }
                rm_models.append(item)
            elif isinstance(op, migrations.DeleteModel):
                # if the migration deletes a table that contains m2m fields, the m2m table will be
                # automatically deleted alongside, so no need to handle it manually
                item = {'model_name': op.name, 'priv_lvl': _priv_lvl }
                rm_models.append(item)
            elif isinstance(op, migrations.AlterModelTable):
                item = {'new_table_name': op.table, 'model_name': op.name, 'priv_lvl': _priv_lvl}
                rename_tables.append(item)
        ## end of for loop
        self._add_models = add_models
        self._rm_models  = rm_models
        self._rename_tables = rename_tables
    ## end of _extract_table_names


    def _execute_raw_sql(self, cursor, sql_pattern, priv_lvl, db_name, table_name, db_user, db_host):
        sql = sql_pattern % (priv_lvl, db_name, table_name, db_user, db_host)
        log_arg = ['renderred_sql', sql]
        _logger.debug(None, *log_arg)
        cursor.execute(sql)

    def _get_add_table_name(self, apps, app_label, item):
        if item['new_table_name'] is None: # db_table not specified
            if item.get('model_to_table_name_required', False) is True:
                fakemodel = apps.get_model(app_label, item['model_name'])
                _table_prefix = fakemodel._meta.db_table
            else:
                _table_prefix = '%s_%s' % (app_label.lower(), item['model_name'].lower())
            if item.get('m2m_fd', None):
                item['new_table_name'] = '%s_%s' % (_table_prefix, item['m2m_fd'].lower())
            else:
                item['new_table_name'] = _table_prefix
        else: # new table name specified
            if item.get('m2m_fd', None):
                item['new_table_name'] = '%s_%s' % (item['new_table_name'].lower(), item['m2m_fd'].lower())
        return item['new_table_name']

    def _grant_table_priv(self, model_list, apps, app_label, cursor, reverse, db_name, db_user, db_host, log_arg):
        done_with_warnings  = False
        if reverse is self._first_run:
            sql_pattern_add = "REVOKE %s ON `%s`.`%s` FROM %s@%s" if reverse is True else "GRANT %s ON `%s`.`%s` TO %s@%s"
            for item  in model_list:
                try: # the pattern below works only in MySQL, TODO, refactor
                    table_name = self._get_add_table_name(apps=apps, app_label=app_label, item=item)
                    log_arg.extend(['add_table', table_name])
                    self._execute_raw_sql(cursor, sql_pattern_add, item['priv_lvl'], db_name, table_name, db_user, db_host)
                except (OperationalError, ProgrammingError) as e:
                    log_arg.extend(['err_msg_grant', self._get_error_msg(e)])
                    done_with_warnings = True
        return done_with_warnings

    def _get_rm_table_name(self, apps, app_label, item):
        fakemodel = apps.get_model(app_label, item['model_name'])
        name_in = fakemodel._meta.db_table
        fd_name = item.get('fd', None)
        if fd_name:
            fd = fakemodel._meta.get_field(fd_name)
            if isinstance(fd, models.ManyToManyField):
                name_out = fd.db_table if fd.db_table else '_'.join([name_in, fd_name])
            else: # removing non m2m fields is NOT related to table deletion
                name_out = None
        else:
            name_out = name_in
        return name_out

    def _revoke_table_priv(self, model_list, apps, app_label, cursor, reverse, db_name, db_user, db_host, log_arg):
        """
        for revoke privilege operation :
        * if not reverse, get `apps` before changing the state and database schema
        * if     reverse, get `apps` after  changing the state and database schema
        """
        done_with_warnings  = False
        if reverse is not self._first_run:
            sql_pattern_del = "GRANT %s ON `%s`.`%s` TO %s@%s" if reverse is True else "REVOKE %s ON `%s`.`%s` FROM %s@%s"
            for item in model_list:
                try:
                    table_name = self._get_rm_table_name(apps=apps, app_label=app_label, item=item)
                    if table_name:
                        log_arg.extend(['rm_table', table_name])
                        self._execute_raw_sql(cursor, sql_pattern_del, item['priv_lvl'], db_name, table_name, db_user, db_host)
                except (OperationalError, LookupError) as e:
                    log_arg.extend(['err_msg_revoke', self._get_error_msg(e)])
                    done_with_warnings = True
        return done_with_warnings


    def _common_handler(self, apps, schema_editor, app_label, reverse=False, **kwargs):
        loglevel = logging.INFO
        log_arg = ['first_run', self._first_run, 'app_label', app_label, 'reverse', reverse,
                'apps', apps, 'to_state_apps', kwargs['to_state_apps']]
        try:
            conn = schema_editor.connection
            caller_db_setup = conn.settings_dict
            migration_caller = '%s@%s:%s' % (caller_db_setup['USER'], caller_db_setup['HOST'], caller_db_setup['PORT'])
            log_arg.extend(['migration_caller', migration_caller,  'conn', conn, 'conn.connection', type(conn.connection)])

            target_db_setup = django_settings.DATABASES[self._db_setup_tag]
            db_name = target_db_setup['NAME']
            db_user = target_db_setup['USER']
            db_host = target_db_setup['HOST']
            log_arg.extend(['db_name', db_name, 'db_user', db_user, 'db_host', db_host])

            assert caller_db_setup['HOST'] == target_db_setup['HOST'], "DB hosts mismatch"
            assert caller_db_setup['PORT'] == target_db_setup['PORT'], "DB server ports mismatch"
            assert caller_db_setup['NAME'] == target_db_setup['NAME'], "database names mismatch"

            done_with_warnings  = False
            with conn.cursor() as cursor:
                apps_for_create = apps if reverse is True  else kwargs['to_state_apps']
                apps_for_delete = apps if reverse is False else kwargs['to_state_apps']  # choose proper apps for delete operation
                log_arg.extend(['apps_for_delete', apps_for_delete])
                done_with_warnings |= self._grant_table_priv(model_list=self._add_models, apps=apps_for_create,
                        cursor=cursor, reverse=reverse, app_label=app_label, db_name=db_name, db_user=db_user,
                        db_host=db_host, log_arg=log_arg)
                done_with_warnings |= self._grant_table_priv(model_list=self._rename_tables, apps=apps_for_create,
                        cursor=cursor, reverse=reverse, app_label=app_label, db_name=db_name, db_user=db_user,
                        db_host=db_host, log_arg=log_arg)
                done_with_warnings |= self._revoke_table_priv(model_list=self._rm_models, apps=apps_for_delete,
                        app_label=app_label, cursor=cursor, reverse=reverse, db_name=db_name, db_user=db_user,
                        db_host=db_host, log_arg=log_arg)
                done_with_warnings |= self._revoke_table_priv(model_list=self._rename_tables, apps=apps_for_delete,
                        app_label=app_label, cursor=cursor, reverse=reverse, db_name=db_name, db_user=db_user,
                        db_host=db_host, log_arg=log_arg)
            if done_with_warnings:
                loglevel = logging.WARNING
            self._first_run = False
        except Exception as e:
            loglevel = logging.ERROR
            log_arg.extend(['err_msg', self._get_error_msg(e)])
        _logger.log(loglevel, None, *log_arg)
    ## end of _common_handler


    def _get_error_msg(self, e):
        e_cls = type(e)
        e_cls_name = '%s.%s' % (e_cls.__module__ , e_cls.__qualname__)
        err_msg = list(map(lambda x: str(x) , e.args))
        err_msg.append(e_cls_name)
        err_msg = ', '.join(err_msg)
        return err_msg
## end of AlterTablePrivilege


# internal call method for AlterTablePrivilege class
def _forward_privilege_setup(apps, schema_editor, app_label, operation, **kwargs):
    operation._common_handler(apps=apps, schema_editor=schema_editor, app_label=app_label,
            reverse=False, **kwargs)


def _backward_privilege_setup(apps, schema_editor, app_label, operation, **kwargs):
    operation._common_handler(apps=apps, schema_editor=schema_editor, app_label=app_label,
            reverse=True, **kwargs)


def monkeypatch_django_migration():
    from django.db.migrations import Migration as DjangoMigration
    origin_init = DjangoMigration.__init__
    def patched_init(self, name, app_label):
        cls = type(self)
        if not hasattr(cls, '_privilege_update_init'):
            self._setup_access_privilege(app_label=app_label)
            cls._privilege_update_init = True
        origin_init(self=self, name=name, app_label=app_label)

    def _setup_access_privilege(self, app_label):
        """
        Automatically grant / revoke all required table-level privileges
        to the database users that are found in django settings.py

        Currently this function grants or revokes full-access privileges (CRUD) of
        each database table at a time. For custom privilege setup such as read-only,
        write only, edit-only... etc , developers need to manually edit the
        auto-generated migration file
        """
        from django.conf import settings as django_settings
        applied_tags = []
        cls = type(self)
        for tag, cfg in django_settings.DATABASES.items():
            reversed_app_label = cfg.get('reversed_app_label', [])
            if app_label in reversed_app_label:
                applied_tags.append(tag)
        for op in cls.operations:
            if isinstance(op, AlterTablePrivilege.ACCEPTED_OPERATIONS) and not hasattr(op, '_priv_lvl'):
                op._priv_lvl = AlterTablePrivilege.PRIVILEGE_MAP['READ_WRITE']
        for tag in applied_tags:
            privilege_update_obj = AlterTablePrivilege( autogen_ops=cls.operations,  db_setup_tag=tag)

    if not hasattr(DjangoMigration.__init__ , '_patched'):
        DjangoMigration.__init__ = patched_init
        DjangoMigration._setup_access_privilege = _setup_access_privilege
        setattr(DjangoMigration.__init__ , '_patched', None)


