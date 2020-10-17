```python
from django.db     import  models

def user_roles_changing_handler(sender, **kwargs):
    ### print("role changing, kwargs : "+ str(kwargs))
    action = kwargs.get('action')
    profile = kwargs.get('instance')
    pk_set = kwargs.get('pk_set')
    using  = kwargs.get('using')

    print("roles "+ action +", pk_set : "+ str(pk_set))
    is_su = profile.is_superuser
    print(" .... is_su: "+ str(is_su))
    if action == "post_add":
        #### account = profile.account
        #### if (su_role_id in pk_set) and account:
        ####     print("superuser rold Id found")
        ####     account.is_superuser = True
        ####     account.save(using=using,)
        pass
    elif action == "post_remove":
        pass
    elif action == "post_clear":
        pass


def user_grps_changing_handler(sender, **kwargs):
    action = kwargs.get('action')
    profile = kwargs.get('instance')
    pk_set = kwargs.get('pk_set')
    using  = kwargs.get('using')
    print("groups "+ action +", pk_set : "+ str(pk_set))
    is_su = profile.is_superuser
    print(" .... is_su: "+ str(is_su))


def user_postsave_handler(sender, **kwargs):
    #### print("user profile post save, kwargs : "+ str(kwargs))
    profile = kwargs.get('instance')
    is_su = profile.is_superuser
    print("user profile post save, is_su: "+ str(is_su))


models.signals.post_save.connect(user_postsave_handler, sender=GenericUserProfile)

models.signals.m2m_changed.connect(user_roles_changing_handler, sender=GenericUserProfile.roles.through)
models.signals.m2m_changed.connect(user_grps_changing_handler, sender=GenericUserProfile.groups.through)


```
