import sys, inspect, argparse
from ecommerce_common.migrations.alembic.config import (
    downgrade_migration,
    auth_provider_upgrade,
    resource_app_upgrade,
)
from settings import test as mig_settings


def auth_migrate_forward(param):
    auth_curr_rev_id = "000002"
    auth_provider_upgrade(
        app_settings=mig_settings, init_round=True, next_rev_id=auth_curr_rev_id
    )


def store_migrate_forward(param):
    dependent_rev_id = (
        None if param.prev_rev_id.lower() == "init" else [param.prev_rev_id]
    )
    kwargs = {
        "app_settings": mig_settings,
        "next_rev_id": param.new_rev_id,
        "new_label": param.new_label,
        "dependent_rev_id": dependent_rev_id,
    }
    resource_app_upgrade(**kwargs)


def migrate_backward(param):
    prev_rev_id = "base" if param.prev_rev_id.lower() == "init" else param.prev_rev_id
    kwargs = {"app_settings": mig_settings, "prev_rev_id": prev_rev_id}
    downgrade_migration(**kwargs)


if __name__ == "__main__":
    toplvl_parser = argparse.ArgumentParser(
        description="dev tools for store-front service maintenance"
    )
    sub_parsers = toplvl_parser.add_subparsers(dest="subcommand")

    subparser = sub_parsers.add_parser("store_migrate_forward")
    subparser.add_argument("--prev-rev-id", required=True, help="Previous revision ID")
    subparser.add_argument("--new-rev-id", required=True, help="New revision ID")
    subparser.add_argument(
        "--new-label", required=True, help="New label associated with the migration"
    )
    subparser = sub_parsers.add_parser("migrate_backward")
    subparser.add_argument("--prev-rev-id", required=True, help="Previous revision ID")

    parsed = toplvl_parser.parse_args()
    curr_mod = sys.modules[__name__]
    members = inspect.getmembers(curr_mod, inspect.isfunction)
    filt = filter(lambda comp: comp[0] == parsed.subcommand, members)
    _, fn = next(filt)
    fn(parsed)
