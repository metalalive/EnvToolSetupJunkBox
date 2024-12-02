import os, shutil
from pathlib import Path
from typing import Optional, List

from alembic import command
from alembic.config import Config
from alembic.util.exc import CommandError

from ecommerce_common.util import format_sqlalchemy_url, get_credential_from_secrets


class ExtendedConfig(Config):
    def __init__(self, *args, template_base_path: Path = None, **kwargs):
        condition = (
            template_base_path
            and template_base_path.exists()
            and template_base_path.is_dir()
        )
        assert condition, "should be existing path to custom template"
        self._template_base_path = template_base_path
        super().__init__(*args, **kwargs)

    def get_template_directory(self) -> str:
        return str(self._template_base_path)

    def set_url(self, db_credential, driver_label):
        url = format_sqlalchemy_url(driver=driver_label, db_credential=db_credential)
        self.set_main_option(name="sqlalchemy.url", value=url)


def _setup_db_credential(base_path: Path, secret_path: str, db_usr_alias):
    _secret_map = {
        db_usr_alias: "backend_apps.databases.%s" % db_usr_alias,
        "usermgt_service": "backend_apps.databases.usermgt_service",
    }
    out_map = get_credential_from_secrets(
        base_path=base_path, secret_path=secret_path, secret_map=_secret_map
    )
    return out_map


def _copy_migration_scripts(src, dst):
    for file_ in src.iterdir():
        if not file_.is_file():
            continue
        if not file_.suffix in (".py",):
            continue
        shutil.copy(str(file_), dst)


DEFAULT_VERSION_TABLE = "alembic_version"


def _init_common_params(
    sys_base_path: Path,
    app_base_path: Path,
    secret_path: Path,
    cfg_filename: str,
    db_usr_alias: str,
):
    cfg_file_path = app_base_path.joinpath(cfg_filename)
    template_base_path = sys_base_path.joinpath(
        "common/python/src/ecommerce_common/migrations/alembic/templates"
    )
    cfg = ExtendedConfig(cfg_file_path, template_base_path=template_base_path)
    migration_base_path = cfg.get_main_option("script_location")
    vpath_delimiter = cfg.get_main_option("version_path_separator")
    mig_version_paths = cfg.get_main_option("version_locations").split(vpath_delimiter)
    mig_version_path = mig_version_paths[0]
    db_credentials = _setup_db_credential(sys_base_path, secret_path, db_usr_alias)
    return cfg, db_credentials, Path(migration_base_path), Path(mig_version_path)


def auth_provider_upgrade(app_settings, init_round: bool, next_rev_id: str):
    db_usr_alias = app_settings.DB_USER_ALIAS
    cfg, db_credentials, migration_base_path, version_path = _init_common_params(
        secret_path=app_settings.SECRETS_FILE_PATH,
        app_base_path=app_settings.APP_BASE_PATH,
        sys_base_path=app_settings.SYS_BASE_PATH,
        db_usr_alias=db_usr_alias,
        cfg_filename="alembic_auth.ini",
    )
    chosen_db_credential = db_credentials[db_usr_alias]
    chosen_db_credential["NAME"] = app_settings.AUTH_DB_NAME
    cfg.set_url(
        db_credential=chosen_db_credential, driver_label=app_settings.AUTH_DRIVER_LABEL
    )
    cfg.set_main_option(name="version_table", value=app_settings.VERSION_TABLE_AUTH_APP)
    cfg.set_main_option(name="app.orm_base", value="skip")
    if init_round:
        command.init(config=cfg, directory=migration_base_path)
    command.revision(
        config=cfg,
        autogenerate=False,
        rev_id="dummyID001",
        message="msg",
        version_path=version_path,
    )
    dummy_path = version_path.joinpath("dummyID001_msg.py")
    os.remove(dummy_path)
    # copy all fixed migration script about authorization / user-management service
    _copy_migration_scripts(src=app_settings.AUTH_MIGRATION_PATH, dst=version_path)
    command.upgrade(config=cfg, revision=next_rev_id)


def resource_app_upgrade(
    app_settings,
    next_rev_id: str,
    new_label: str,
    dependent_rev_id: Optional[List[str]],
):
    init_round = dependent_rev_id is None
    db_usr_alias = app_settings.DB_USER_ALIAS
    orm_base_cls_path = app_settings.ORM_BASE_CLASSES
    cfg, db_credentials, migration_base_path, version_path = _init_common_params(
        secret_path=app_settings.SECRETS_FILE_PATH,
        sys_base_path=app_settings.SYS_BASE_PATH,
        app_base_path=app_settings.APP_BASE_PATH,
        db_usr_alias=db_usr_alias,
        cfg_filename="alembic_app.ini",
    )
    chosen_db_credential = db_credentials[db_usr_alias]
    chosen_db_credential["NAME"] = app_settings.DB_NAME
    cfg.set_url(
        db_credential=chosen_db_credential, driver_label=app_settings.DRIVER_LABEL
    )
    cfg.set_main_option(name="version_table", value=DEFAULT_VERSION_TABLE)
    cfg.set_main_option(name="app.orm_base", value=",".join(orm_base_cls_path))
    if init_round:
        command.init(config=cfg, directory=migration_base_path)
    # ------------------
    result = command.revision(
        config=cfg,
        message=new_label,
        autogenerate=True,
        rev_id=next_rev_id,
        depends_on=dependent_rev_id,
        version_path=version_path,
    )
    assert result.revision == next_rev_id
    command.upgrade(config=cfg, revision=next_rev_id)


def downgrade_migration(app_settings, prev_rev_id: str):
    if prev_rev_id == "base":
        _downgrade_auth_migration(app_settings, prev_rev_id)
    _downgrade_app_migration(app_settings, prev_rev_id)
    basepath = app_settings.APP_BASE_PATH.joinpath("migrations")
    if prev_rev_id == "base" and basepath.exists():
        shutil.rmtree(basepath)


def _downgrade_app_migration(app_settings, prev_rev_id: str) -> Path:
    db_usr_alias = app_settings.DB_USER_ALIAS
    orm_base_cls_path = app_settings.ORM_BASE_CLASSES
    cfg, db_credentials, _, _ = _init_common_params(
        secret_path=app_settings.SECRETS_FILE_PATH,
        app_base_path=app_settings.APP_BASE_PATH,
        sys_base_path=app_settings.SYS_BASE_PATH,
        db_usr_alias=db_usr_alias,
        cfg_filename="alembic_app.ini",
    )
    chosen_db_credential = db_credentials[db_usr_alias]
    chosen_db_credential["NAME"] = app_settings.DB_NAME
    cfg.set_url(
        db_credential=chosen_db_credential, driver_label=app_settings.DRIVER_LABEL
    )
    cfg.set_main_option(
        name="version_table", value=DEFAULT_VERSION_TABLE
    )  # must not be NULL or empty string
    cfg.set_main_option(name="app.orm_base", value=",".join(orm_base_cls_path))
    try:
        command.downgrade(config=cfg, revision=prev_rev_id)
    except CommandError as e:
        pos = e.args[0].lower().find("path doesn't exist")
        if pos < 0:
            raise


def _downgrade_auth_migration(app_settings, prev_rev_id: str):
    # TODO , improve the design, authorization database server might
    # be different from the database server for this application
    pass
    # db_usr_alias = app_settings.DB_USER_ALIAS
    # cfg, db_credentials, _, _ = _init_common_params(
    #     secret_path=app_settings.SECRETS_FILE_PATH,
    #     app_base_path=app_settings.APP_BASE_PATH,
    #     sys_base_path=app_settings.SYS_BASE_PATH,
    #     db_usr_alias=db_usr_alias,
    #     cfg_filename="alembic_auth.ini",
    # )
    # chosen_db_credential = db_credentials[db_usr_alias]
    # chosen_db_credential["NAME"] = app_settings.AUTH_DB_NAME
    # cfg.set_url(
    #     db_credential=chosen_db_credential, driver_label=app_settings.DRIVER_LABEL
    # )
    # cfg.set_main_option(name="version_table", value=app_settings.VERSION_TABLE_AUTH_APP)
    # cfg.set_main_option(name="app.orm_base", value="skip")
    # try:
    #     command.downgrade(config=cfg, revision=prev_rev_id)
    # except CommandError as e:
    #     pos = e.args[0].lower().find("path doesn't exist")
    #     if pos < 0:
    #         raise
