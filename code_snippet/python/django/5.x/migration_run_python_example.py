from django.db import migrations
from django.db.models.query import QuerySet
from django.db.migrations.state import StateApps
from django.db.backends.base.schema import BaseDatabaseSchemaEditor

from ecommerce_common.models.enums.django import AppCodeOptions
from typing import Dict,List

_PermDataType = Dict[str,List[Dict[str,str]]]

class RemoteAppMigration:
    def __init__(self, app: StateApps, db_alias:str):
        self._app = app
        self._db_alias = db_alias

    def create_content_type(self, app_label:str, modellabels:List[str] ) -> Dict:
        ctype_cls = self._app.get_model("contenttypes", "contenttype")
        qset = ctype_cls.objects.using(self._db_alias)
        ctype_objs = {
            m: ctype_cls(app_label=app_label, model=m) for m in modellabels
        }
        qset.bulk_create(ctype_objs.values())
        return ctype_objs

    def create_permissions(self, app_label:str, ctype_objs:Dict, data: _PermDataType):
        authperm_cls = self._app.get_model("auth", "permission")
        qset = authperm_cls.objects.using(self._db_alias)
        for mlabel, permdata in data.items():
            ctype_obj = ctype_objs[mlabel]
            perm_objs = [
                authperm_cls(name=d['name'], codename=d['code'], content_type=ctype_obj)
                for d in permdata
            ]
            qset.bulk_create(perm_objs)

    def create_quota_material(self, app_code:int, mat_codes:List[int]):
        quotamat_cls = self._app.get_model("user_management", "quotamaterial")
        qset = quotamat_cls.objects.using(self._db_alias)
        quotamat_objs = [quotamat_cls(app_code=app_code, mat_code=m) for m in mat_codes]
        qset.bulk_create(quotamat_objs)

    def delete_quota_material(self, app_code:int, mat_codes:List[int]):
        quotamat_cls = self._app.get_model("user_management", "quotamaterial")
        qset = quotamat_cls.objects.using(self._db_alias)
        qset.filter(app_code=app_code, mat_code__in=mat_codes).delete()

    def delete_permissions(self, app_label:str, data: _PermDataType):
        authperm_cls = self._app.get_model("auth", "permission")
        ctype_cls = self._app.get_model("contenttypes", "contenttype")
        qset1 = ctype_cls.objects.using(self._db_alias)
        qset2 = authperm_cls.objects.using(self._db_alias)
        for mlabel, permdata in data.items():
            ctype_obj = qset1.get(app_label=app_label, model=mlabel)
            extract_codes = [d['code'] for d in permdata  ]
            qset2.filter(codename__in=extract_codes, content_type=ctype_obj ).delete()

    def delete_content_type(self, app_label:str, modellabels:List[str] ):
        ctype_cls = self._app.get_model("contenttypes", "contenttype")
        qset = ctype_cls.objects.using(self._db_alias)
        qset.filter(app_label=app_label, model__in=modellabels).delete()

model_permissions = {
    "tag": [
        {"code": "add_producttag", "name": "Can add product tag"},
        {"code": "change_producttag", "name": "Can change product tag"},
        {"code": "delete_producttag", "name": "Can delete product tag"},
    ],
    "attributelabel": [
        {"code": "add_attributelabel", "name": "Can add product attribute label"},
        {"code": "change_attributelabel", "name": "Can change product attribute label"},
        {"code": "delete_attributelabel", "name": "Can delete product attribute label"},
    ],
    "saleableitem": [
        {"code": "add_saleableitem", "name": "Can add saleable item"},
        {"code": "change_saleableitem", "name": "Can change saleable item"},
        {"code": "delete_saleableitem", "name": "Can delete saleable item"},
        {"code": "view_saleableitem", "name": "Can view saleable item"},
    ],
    "saleablepackage": [
        {"code": "add_saleablepackage", "name": "Can add saleable package"},
        {"code": "change_saleablepackage", "name": "Can change saleable package"},
        {"code": "delete_saleablepackage", "name": "Can delete saleable package"},
        {"code": "view_saleablepackage", "name": "Can view saleable package"},
    ],
}
app_label = AppCodeOptions.product.name
app_code = AppCodeOptions.product.value
material_codes = [2, 3]


def forwards(app: StateApps, schema_editor: BaseDatabaseSchemaEditor):
    db_alias = schema_editor.connection.alias
    mig = RemoteAppMigration(app, db_alias)
    ctype_objs = mig.create_content_type(app_label, model_permissions.keys())
    mig.create_permissions(app_label, ctype_objs, data=model_permissions)
    mig.create_quota_material(app_code, material_codes)


def backwards(app: StateApps, schema_editor: BaseDatabaseSchemaEditor):
    db_alias = schema_editor.connection.alias
    mig = RemoteAppMigration(app, db_alias)
    mig.delete_quota_material(app_code, material_codes)
    mig.delete_permissions(app_label, data=model_permissions)
    mig.delete_content_type(app_label, model_permissions.keys())
    # raise NotImplementedError()


class Migration(migrations.Migration):
    dependencies = [("user_management", "0002_rawsqls")]

    operations = [migrations.RunPython(forwards, backwards, atomic=True)]
