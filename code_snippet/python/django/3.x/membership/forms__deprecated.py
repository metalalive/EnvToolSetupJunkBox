from datetime import datetime, timezone, timedelta

from django.forms    import  Form, ModelForm, IntegerField, MultipleChoiceField, HiddenInput, BooleanField, ALL_FIELDS, CharField
from django.core     import  validators
from django.contrib  import  auth
from django.db       import  IntegrityError, transaction, models

from django.core.exceptions   import NON_FIELD_ERRORS, ValidationError, ObjectDoesNotExist
from django.forms.models      import modelformset_factory
from django.forms.utils       import ErrorList, ErrorDict
from django.forms.formsets    import MAX_NUM_FORM_COUNT, MIN_NUM_FORM_COUNT, TOTAL_FORM_COUNT
from django.utils.deconstruct import deconstructible
from django.contrib.contenttypes.models import ContentType

from common.forms.forms       import  ExtendedModelForm, ExtendedBaseForm
from common.forms.formsets    import  ExtendedBaseFormSet, ClosureTableFormsetMixin, ExtendedBaseModelFormSet
from common.forms.validators  import  SelectIDsExistValidator
from location.models          import  Location

from .models  import GenericUserGroup, GenericUserGroupClosure
from .models  import GenericUserProfile, EmailAddress, PhoneNumber, UserEmailAddress, UserPhoneNumber, UserLocation
from .models  import AuthUserResetRequest, GenericUserAuthRelation

class GenericUserGroupForm(ExtendedModelForm):
    """ extend ModelForm class to do extra check on meta data if used in user applications """
    class Meta:
        model = GenericUserGroup
        fields = ALL_FIELDS
        #exclude = ['id',] # Don't use meta class since we'd like to dynamically  exclude ID field only in edit view

    def save(self, commit=True, formset=None):
        return super().save(commit=commit, formset=formset, form=self)



class GenericUserProfileForm(ExtendedModelForm):
    class Meta:
        model = GenericUserProfile
        exclude = ['max_num_addr', 'max_num_phone', 'max_num_email', 'max_bookings', 'max_entry_waitlist', ]

    def __init__(self, data=None, files=None, auto_id='id_%s', prefix=None, instance=None,
                 initial=None, error_class=ErrorList, label_suffix=None, empty_permitted=False,
                 use_required_attribute=None,  renderer=None ):
        super(GenericUserProfileForm, self).__init__(data=data, files=files, auto_id=auto_id,
                prefix=prefix, initial=initial, error_class=error_class , label_suffix=label_suffix,
                empty_permitted=empty_permitted, use_required_attribute=use_required_attribute,
                instance=instance, renderer=renderer)
        from location.forms import  LocationForm
        self.subforms = {'email':    {'model_cls':EmailAddress, 'relate_model_cls':UserEmailAddress,
                         'relate_field_name':UserEmailAddress.email.field.name, 'form_cls':EmailAddressForm,
                         'formset_cls':CommonUserContactFormset , 'obj':None, 'quota':0, 'prefix':EmailAddressForm.prefix,},

                         'phone':    {'model_cls':PhoneNumber, 'relate_model_cls':UserPhoneNumber,
                         'relate_field_name':UserPhoneNumber.phone.field.name, 'form_cls': PhoneNumberForm,
                         'formset_cls':CommonUserContactFormset , 'obj':None, 'quota':0, 'prefix':PhoneNumberForm.prefix, },

                         'location': {'model_cls':Location, 'relate_model_cls': UserLocation,
                         'relate_field_name':UserLocation.address.field.name, 'form_cls': LocationForm,
                         'formset_cls':CommonUserContactFormset , 'obj':None, 'quota':0, 'prefix':LocationForm.prefix, },
                         }
        #### print("[GenericUserProfileForm] init instance : "+ str(self.instance))
        #### print("[GenericUserProfileForm] prefix : "+ str(self.prefix))
        for v in self.subforms.values():
            self._init_subforms(props=v, initial=initial)
        self.subform_quotas = UserQuotaForm(data=self.data, instance=self.instance,
                    prefix=self.add_prefix(UserQuotaForm.prefix))
        if self.instance.pk is None:
            self.auth_rst_req = AuthUserResetRequestForm(prefix=self.prefix, profile=self.instance,
                            data=self.data,  email_subforms=self.subforms['email']['obj'])


    # handle extensible subforms using email/phone/location formset in each user profile form
    def _init_subforms(self, props, initial=None):
        qset = props['model_cls'].objects.none()
        if self.is_bound:
            field_name = self.add_prefix("-".join([props['prefix'], MAX_NUM_FORM_COUNT]))
            max_num    = int(self.data.get(field_name, 0))
            field_name = self.add_prefix("-".join([props['prefix'], MIN_NUM_FORM_COUNT]))
            min_num    = int(self.data.get(field_name, 0))
            field_name = self.add_prefix("-".join([props['prefix'], TOTAL_FORM_COUNT]))
            extra_num = int(self.data.get(field_name, 0))
        elif self.instance.pk: # TODO: finish implementation
            user_type = ContentType.objects.get_for_model(self.instance)
            condition = models.Q(user_type=user_type) and models.Q(user_id=self.instance.pk)
            qset = props['relate_model_cls'].objects.filter(condition)
            max_num   = len(qset)
            min_num   = max_num
            extra_num = 0
            pk_list = [int(getattr(q, props['relate_field_name']).pk) for q in qset]
            qset = props['model_cls'].objects.filter(models.Q(id__in=pk_list))
        else :
            return
        #### print("subform email form number, max_num:"+ str(max_num) +", extra_num:"+ str(extra_num))
        if max_num <= 0 or min_num <= 0:
            return # figure out how to use initial
        formset_cls = modelformset_factory(props['model_cls'], form=props['form_cls'], extra=extra_num,
                        max_num=max_num, min_num=min_num, formset=props['formset_cls'])
        props['obj'] = formset_cls(data=self.data, prefix=self.add_prefix(props['prefix']), queryset=qset)


    def clean(self):
        cleaned_data = super(GenericUserProfileForm, self).clean()
        # validate quota subform first
        self.subform_quotas.errors # cannot call BaseFrom.clean() directly
        self.subform_quotas.copy_errors(dst=self._errors)
        if hasattr(self, 'auth_rst_req'):
            self._errors.update(self.auth_rst_req.errors)
        cleaned_data_quota = self.subform_quotas.clean()
        #### print("cleaned_data : "+ str(cleaned_data))
        #### print("cleaned_data_quota : "+ str(cleaned_data_quota))
        # validate rest of subforms
        self.subforms['email']['quota']    = cleaned_data_quota.get('max_num_email', -1)
        self.subforms['phone']['quota']    = cleaned_data_quota.get('max_num_phone', -1)
        self.subforms['location']['quota'] = cleaned_data_quota.get('max_num_addr', -1)
        for v in self.subforms.values():
            self._clean_subforms(props=v)
        return cleaned_data


    def _clean_subforms(self, props):
        if (props['obj'] is None) or (props['quota'] < 0):
            return
        props['obj'].quota = props['quota']
        props['obj'].clean() # start subform validation
        props['obj'].copy_errors(dst=self._errors)


    @transaction.atomic
    def save(self, commit=True):
        """ save user-profile forms and all the subforms """
        # copy part of cleaned data from quota form
        #### print("self.cleaned_data : "+ str(self.cleaned_data))
        super().save(commit=commit)
        self.subform_quotas.save(commit=commit)
        for k, v in self.subforms.items():
            if v['obj']:
               v['obj'].profile = self.instance
               v['obj'].save(commit=commit)

    def postprocess_if_valid(self, **kwargs):
        profile_obj = self.save()
        if hasattr(self, 'auth_rst_req'):
            self.auth_rst_req.save(**kwargs) # activate account only on profile creation



# try different way of subform validation, better approach would be modelform
# then separate all quota attributes from user group / user profile model.
class UserQuotaForm(ExtendedModelForm):
    class Meta:
        model = GenericUserProfile
        # fields = ['max_num_addr', 'max_num_phone', 'max_num_email', 'max_bookings', 'max_entry_waitlist', ]
        fields = []
    prefix = "quotas"


class  CommonUserContactForm(ExtendedModelForm):
    relate_model = None
    relate_field = None

    @property
    def profile(self):
        return self._profile

    @profile.setter
    def profile(self, ref):
        self._profile = ref

    def save(self, commit=True, **kwargs):
        if self.relate_model is None or self.relate_field is None or not hasattr(self, '_profile'):
            raise ValueError('`relate_model` and `relate_field` must be specified.')
        #### print(self.prefix +", "+ str(self._profile)+", cleaned_data : "+ str(self.cleaned_data))
        super().save()
        kwargs = {self.relate_field : self.instance}
        _relate = self.relate_model.objects.create(**kwargs, user_id=self._profile.pk,
                    user_type = ContentType.objects.get_for_model(self._profile))
        # TODO: currrently only consider creation case, what about edit case ?



class  EmailAddressForm(CommonUserContactForm):
    class Meta:
        model = EmailAddress
        fields = ALL_FIELDS
    prefix = 'email'
    relate_model = UserEmailAddress
    relate_field = 'email'


class  PhoneNumberForm(CommonUserContactForm):
    class Meta:
        model = PhoneNumber
        fields = ALL_FIELDS
    prefix = 'phone'
    relate_model = UserPhoneNumber
    relate_field = 'phone'


class  CommonUserContactFormset(ExtendedBaseModelFormSet):
    @property
    def profile(self):
        return self.forms[0].profile

    @profile.setter
    def profile(self, ref):
        for form in self.forms:
            form.profile = ref

    def prepare_formset_wide_validators(self):
        err_msg = ["number of forms = ", str(len(self.forms))," exceeds the given quota = ", str(self.quota)]
        validator_cls = validators.MinValueValidator
        init_kwargs = {'limit_value':len(self.forms), 'message': "".join(err_msg)}
        call_kwargs = {'value': self.quota}
        self.add_validator(validator_cls, init_kwargs, call_kwargs)


@deconstructible
class EnableAccountValidator:
    """ In this implementation, email is required to enable user account """
    def __init__(self, email_provided):
        self.email_provided = email_provided
        self.message = "email is required to enable user account."

    def __call__(self, value):
        if self.email_provided is None and value is True :
            raise ValidationError(self.message)


class AuthUserResetRequestForm(ExtendedModelForm):
    class Meta:
        model = AuthUserResetRequest
        fields = ALL_FIELDS

    enable_auth = BooleanField(required=False)

    def __init__(self, data=None, files=None, auto_id='id_%s', prefix=None, profile=None,
                 initial=None, error_class=ErrorList, label_suffix=None, email_subforms=None,
                 empty_permitted=False, use_required_attribute=None, renderer=None):
        self._profile = profile
        try:
            if profile:
                instance = self._meta.model.objects.get(profile=profile.pk)
            else:
                instance = None
        except self._meta.model.DoesNotExist as e:
            instance = None
        super().__init__(data=data, files=files, auto_id=auto_id,
                prefix=prefix, initial=initial, error_class=error_class , label_suffix=label_suffix,
                empty_permitted=empty_permitted, use_required_attribute=use_required_attribute,
                instance=instance, renderer=renderer)
        # TODO: rethink, is it essential to feed email_subforms ? email can be fetched from profile
        validator   = EnableAccountValidator(email_provided=email_subforms)
        enable_auth = self.fields['enable_auth']
        enable_auth.validators.append(validator)

    @transaction.atomic
    def save(self, commit=True, **kwargs):
        if not self.cleaned_data['enable_auth']:
            print("skip user account activation...")
        else:
            super().save(commit=commit, profile=self._profile)
            # send mail only when the token is newly created, maybe do this asynchronously ?
            activate_url = [kwargs['hostname'], kwargs['res_path'], self.instance.token]
            kwargs = AuthUserResetRequestForm.create_mail_content(activate_url)
            self._profile.send_mail(**kwargs)
        return self.instance
        #### print("instance token to user mail : "+ str(self.instance.token))

    @classmethod
    def create_mail_content(self, activate_url):
        full_url = "/".join(activate_url)
        message = "Your login account is created at {time}, activate it by clicking the link {full_url} in {time_limit} minutes"
        message = message.format(time=str(datetime.now(timezone.utc)), full_url=full_url,
                    time_limit=str(AuthUserResetRequest.MAX_TOKEN_VALID_TIME/60))
        subject = "please activate your account (do not reply)"
        out = {'message': message, 'subject': subject, 'html_message':None}
        return out




class GenericUserGroupFormset(ExtendedBaseFormSet, ClosureTableFormsetMixin):

    def init_IDs_extension(self, model_qset, IDs, initial):
        """ set up initial value that will go to the formset """
        idx = 0
        for grp in model_qset:
            initial[idx]['name'] = grp.name
            idx += 1
        self.init_parent_IDs(model_qset, IDs, initial, closure_model=GenericUserGroupClosure )
        self._init_IDs_permission_selected(model_qset, IDs, initial)


    def _init_IDs_permission_selected(self, model_qset, IDs, initial):
        idx = 0
        for grp in model_qset:
            try: # TODO : validate
                # accessing  auth_roles might cause exception, because not every generic
                # user group has corresponding group in django.contrib.auth application.
                roles = list(grp.auth_roles.all())
                permissions = []
                for role in roles:
                    permissions += list(role.permissions.all())
                initial[idx]['permissions_selected'] = [(p.id, p.name) for p in permissions]
            except  ObjectDoesNotExist:
                initial[idx]['permissions_selected'] = []
            idx += 1

    def add_fields(self, form, index):
        """
        Besides those fields originally defined in the model, this formset class also
        adds extra form fields for parent group selection
        """
        super().add_fields(form, index)
        # locally import this specific view class, to avoid circular imports
        if self.action_type is ExtendedBaseFormSet.ACTION_TYPE_CREATE:
            self._add_new_parents_field(form)
        elif self.action_type  is ExtendedBaseFormSet.ACTION_TYPE_EDIT:
            self._add_ugid_field(form)
        self._add_existing_parents_field(form)
        self._add_permission_field(form)


    def _add_permission_field(self, form):
        permissions_selected = []
        validators_list = []
        if form.is_bound:
            field_name = "-".join([form.prefix, 'permissions_selected'])
            permissions_selected = self.data.getlist(field_name)
            ### print("".join(['permissions_selected: ', str(permissions_selected), ]))
            if not permissions_selected in validators.EMPTY_VALUES:
                permissions_selected = [int(i) for i in permissions_selected]
                id_union = models.Q(id__in=permissions_selected)
                qset = auth.models.Permission.objects.filter(id_union)
                permissions_selected = [(int(q.id) , q.name) for q in qset]
                validator = SelectIDsExistValidator(queryset=qset)
                validators_list.append(validator)
        elif form.initial.get('permissions_selected', None):
            permissions_selected = form.initial['permissions_selected']
        form.fields["permissions_selected"] = MultipleChoiceField(required=False, \
                choices=tuple(permissions_selected), validators=tuple(validators_list))


    def prepare_formset_wide_validators(self):
        """
        2 cases to validate at here:
        * ID field of each form (in a single formset) must be distinct from each other
        * Cycle detection among distinct (group hierarchy) trees
        """
        validator_cls, init_kwargs, call_kwargs = self._prepare_distinct_IDs_validators()
        self.add_validator(validator_cls, init_kwargs, call_kwargs)
        validator_cls, init_kwargs, call_kwargs = self.prepare_cycle_detection_validators(
                                                    forms=self.forms, model_cls=GenericUserGroupClosure)
        self.add_validator(validator_cls, init_kwargs, call_kwargs)


    def _add_existing_parents_field(self, form):
        """ add extra validator to exist_parent, ONLY on update view """
        validators_list = []
        ugid = 0
        if form.is_bound:
            validators_list.append(SelectIDsExistValidator(model_cls=GenericUserGroup))
            if self.action_type is ExtendedBaseFormSet.ACTION_TYPE_EDIT:
                ugid = int(form.data["".join([form.prefix, "-", "id"])])
                validators_list.append(self._prepare_parent_validator(root_id=ugid, model_cls=GenericUserGroupClosure))
        super()._add_existing_parents_field(form=form, validators_list=validators_list)


    def _add_ugid_field(self, form):
        # TODO: any better way to hide the IDs on client side ?
        form.fields["id"] = IntegerField(required=True, widget=HiddenInput())


    def _insert_auth_group(self, form):
        permissions_selected = form.cleaned_data['permissions_selected']
        if not permissions_selected in validators.EMPTY_VALUES:
            # create the same group name in auth group table
            authgroup = auth.models.Group.objects.create(name=form.cleaned_data['name'])
            # insert to "auth group" and "generic user group" relation table
            from .models  import GenericGroupAuthRelation
            relation = GenericGroupAuthRelation.objects.create(authgroup=authgroup, gusergroup=form.instance)
            # insert to "auth group" and "auth permission" relation table
            qset = auth.models.Permission.objects.filter(models.Q(id__in=permissions_selected))
            authgroup.permissions.add(*qset)


    def _update_auth_group(self, form):
        permissions_selected_new = [int(v) for v in form.cleaned_data['permissions_selected']]
        permissions_selected_old = [v0 for v0, v1 in form.initial['permissions_selected']]
        if set(permissions_selected_new) != set(permissions_selected_old):
            # print("".join(["(_update_auth_group) form.instance.id : ", str(form.instance.id), ])  )
            # print("".join(["permissions_selected_old : ", str(permissions_selected_old), ])  )
            # print("".join(["permissions_selected_new : ", str(permissions_selected_new), ])  )
            if permissions_selected_old in validators.EMPTY_VALUES:
                # create the same group name in auth group table
                from .models  import GenericGroupAuthRelation
                authgroup = auth.models.Group.objects.create(name=form.cleaned_data['name'])
                relation = GenericGroupAuthRelation.objects.create(authgroup=authgroup, gusergroup=form.instance)
            else:
                relation  = form.instance.auth_relate
                authgroup = relation.authgroup
            if permissions_selected_new in validators.EMPTY_VALUES:
                # delete the same group in auth group table
                relation.delete(hard=True)
                # all permissions of the group should be automatically deleted because they are foreign keys
                authgroup.permissions.clear()
                authgroup.delete(hard=True)
            else:
                qset = auth.models.Permission.objects.filter(models.Q(id__in=permissions_selected_new))
                authgroup.permissions.set(list(qset))


    # multiple new models will be saved by calling save(), to ensure integrity of ORM operations,
    # transcation.atomic() must be used
    @transaction.atomic
    def insert(self):
        sorted_forms = self.get_sorted_insertion_forms()
        for form in sorted_forms:
            form.save(formset=self) # save forms in sorted order
            # insert to auth group table, auth permission table, if any of permissions is selected
            self._insert_auth_group(form=form)
        #### raise IntegrityError(e)


    def update(self):
        """
        This function calls internal prepare_update_nodes() to get update lists, which will be used
        for bulked CRUD operations to the closure table
        """
        closure_model=GenericUserGroupClosure
        sorted_forms = self.get_sorted_update_forms(closure_model=closure_model)
        with transaction.atomic():
            for form in sorted_forms:
                form.save(formset=self)
                self._update_auth_group(form)
            self.clean_deleted_closure_path()
        #### raise IntegrityError as e:

#### end of GenericUserGroupFormset






    #### print("[GenericUserProfileForm] init groups : "+ str(data.get('user-profile-form-0-groups')))
    ####    item['cls'] = generic_inlineformset_factory( UserEmailAddress, form=ExtendedModelForm,
    ####                      formset=BaseGenericInlineFormSet, ct_field='user_type', fk_field='user_id',
    ####                      extra=extra_num, max_num=max_num, formfield_callback=None,)
    ####    item['obj'] = item['cls'](data=self.data, prefix=prefix, instance=self.instance, queryset=None)

