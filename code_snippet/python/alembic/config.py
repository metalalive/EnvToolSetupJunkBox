import os
import shutil
from pathlib import Path
from typing import Optional, List 

from alembic import command
from alembic.config import Config
from alembic.util.exc import CommandError

from common.util.python import format_sqlalchemy_url
from common.util.python import get_credential_from_secrets

"""
This module tries to create multiple bases to different databases and upgrade to each of them,
the code here does NOT work because of Alembic's restriction on revision check.
Check out the answer I wrote at :
https://stackoverflow.com/a/76947458/9853105
"""

class ExtendedConfig(Config):
    def __init__(self, *args, template_base_path:Path=None, **kwargs):
        condition = template_base_path and template_base_path.exists() and template_base_path.is_dir()
        assert condition , 'should be existing path to custom template'
        self._template_base_path = template_base_path
        super().__init__(*args, **kwargs)

    def get_template_directory(self) -> str:
        return str(self._template_base_path)

    def set_url(self, db_credential, driver_label):
        url = format_sqlalchemy_url(driver=driver_label, db_credential=db_credential)
        self.set_main_option(name='sqlalchemy.url', value=url)



def _setup_db_credential(secret_path, db_usr_alias):
    _secret_map = {
        db_usr_alias      : 'backend_apps.databases.%s' % db_usr_alias,
        'usermgt_service' : 'backend_apps.databases.usermgt_service' ,
    }
    out_map = get_credential_from_secrets(base_folder='staff_portal',
            secret_path=secret_path, secret_map=_secret_map )
    return out_map


def _copy_migration_scripts(src, dst):
    for file_ in src.iterdir():
        if not file_.is_file():
            continue
        if not file_.suffix in ('.py',):
            continue
        shutil.copy(str(file_), dst)


DEFAULT_VERSION_TABLE = 'alembic_version'

def _init_common_params(app_base_path, secret_path, db_usr_alias):
    migration_base_path = app_base_path.joinpath('migrations')
    cfg_file_path  = app_base_path.joinpath('alembic.ini')
    template_base_path = app_base_path.parent.joinpath('migrations/alembic/templates')
    alembic_cfg = ExtendedConfig(cfg_file_path, template_base_path=template_base_path)
    db_credentials = _setup_db_credential(secret_path, db_usr_alias)
    return migration_base_path, alembic_cfg, db_credentials


def auth_provider_upgrade (app_settings, init_round:bool, next_rev_id:str) :
    db_usr_alias = app_settings.DB_USER_ALIAS
    migration_base_path, alembic_cfg, db_credentials = _init_common_params(
            secret_path=app_settings.SECRETS_FILE_PATH,
            app_base_path=app_settings.APP_BASE_PATH,
            db_usr_alias=db_usr_alias )
    mig_script_path = migration_base_path.joinpath('versions/app')
    testhd = 'head' if init_round else 'llama@head'
    command.revision( config=alembic_cfg,  autogenerate=False,
            branch_label='llama',  head=testhd,
            rev_id='dummyID001', message='msg',  version_path=mig_script_path )
    dummy_path = mig_script_path.joinpath('dummyID001_msg.py')
    os.remove(dummy_path)
    # copy all fixed migration script about authorization / user-management service
    _copy_migration_scripts(src=app_settings.AUTH_MIGRATION_PATH, dst=mig_script_path)
    chosen_db_credential = db_credentials[db_usr_alias]
    chosen_db_credential['NAME'] = app_settings.AUTH_DB_NAME
    alembic_cfg.set_url(db_credential=chosen_db_credential, driver_label=app_settings.AUTH_DRIVER_LABEL)
    alembic_cfg.set_main_option(name='version_table', value=app_settings.VERSION_TABLE_AUTH_APP)
    alembic_cfg.set_main_option(name='app.orm_base', value='skip')
    command.upgrade(config=alembic_cfg, revision=next_rev_id)


def resource_app_upgrade ( app_settings, next_rev_id:str, new_label:str,
        dependent_rev_id:Optional[List[str]] ) :
    init_round = dependent_rev_id is None
    db_usr_alias = app_settings.DB_USER_ALIAS
    orm_base_cls_path = app_settings.ORM_BASE_CLASSES
    migration_base_path, alembic_cfg, db_credentials = _init_common_params(
            secret_path=app_settings.SECRETS_FILE_PATH,
            app_base_path=app_settings.APP_BASE_PATH,
            db_usr_alias=db_usr_alias )
    mig_script_path = migration_base_path.joinpath('versions/app')
    chosen_db_credential = db_credentials[db_usr_alias]
    chosen_db_credential['NAME'] = app_settings.DB_NAME
    alembic_cfg.set_url(db_credential=chosen_db_credential, driver_label=app_settings.DRIVER_LABEL)
    alembic_cfg.set_main_option(name='version_table', value=DEFAULT_VERSION_TABLE)
    alembic_cfg.set_main_option(name='app.orm_base', value=','.join(orm_base_cls_path))
    if init_round:
        command.init(config=alembic_cfg, directory=migration_base_path)
        result = command.revision( config=alembic_cfg,  autogenerate=False,
            rev_id='app0000', message='msg',  version_path=mig_script_path )
        assert result.revision == 'app0000'
        dependent_rev_id = [result.revision]
        command.upgrade(config=alembic_cfg, revision=result.revision)
    # ------------------
    testhd = 'head' if init_round else 'giblee@head'
    result = command.revision( config=alembic_cfg, message=new_label, autogenerate=True,
            branch_label='giblee',  head=testhd,  splice=init_round,
            rev_id= next_rev_id,  depends_on=dependent_rev_id, version_path=mig_script_path )
    assert result.revision == next_rev_id
    command.upgrade(config=alembic_cfg, revision=next_rev_id)



def downgrade_migration(app_settings, prev_rev_id:str) :
    db_usr_alias = app_settings.DB_USER_ALIAS
    orm_base_cls_path = app_settings.ORM_BASE_CLASSES
    migration_base_path, alembic_cfg, db_credentials = _init_common_params(
            secret_path=app_settings.SECRETS_FILE_PATH,
            app_base_path=app_settings.APP_BASE_PATH,
            db_usr_alias=db_usr_alias
        )
    alembic_cfg.set_main_option(name='app.orm_base', value=','.join(orm_base_cls_path))
    chosen_db_credential = db_credentials[db_usr_alias]
    try: # ------------------
        if prev_rev_id == 'base':
            chosen_db_credential['NAME'] = app_settings.AUTH_DB_NAME
            alembic_cfg.set_url(db_credential=chosen_db_credential, driver_label=app_settings.DRIVER_LABEL)
            alembic_cfg.set_main_option(name='version_table', value=app_settings.VERSION_TABLE_AUTH_APP)
            command.downgrade(config=alembic_cfg, revision=prev_rev_id)
        # ------------------
        chosen_db_credential['NAME'] = app_settings.DB_NAME
        alembic_cfg.set_url(db_credential=chosen_db_credential, driver_label=app_settings.DRIVER_LABEL)
        alembic_cfg.set_main_option(name='version_table', value=DEFAULT_VERSION_TABLE) # must not be NULL or empty string
        command.downgrade(config=alembic_cfg, revision=prev_rev_id)
    except CommandError as e:
        pos = e.args[0].lower().find('path doesn\'t exist')
        if pos < 0:
            raise
    if prev_rev_id == 'base' and migration_base_path.exists():
        shutil.rmtree(migration_base_path)

