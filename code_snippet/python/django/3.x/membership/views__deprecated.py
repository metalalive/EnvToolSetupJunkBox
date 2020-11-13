

class UserGroupsCreateView(CreateView):
    form_class = GenericUserGroupForm
    template_name = UserMgtCfg.template_name[__qualname__]
    success_url_resource = UserMgtCfg.success_url[__qualname__]
    _400_template_name   = UserMgtCfg.template_name['Http400BadRequestView']

   def get(self, request, *args, **kwargs):
       self.object = None
       try:
           total_num_forms = int(request.GET['total_num_forms'])
           total_num_forms = min(total_num_forms, MAX_NUM_FORM)
       except (ValueError, MultiValueDictKeyError) as e:
           total_num_forms = DEFAULT_NUM_FORM
       #### print("total_num_forms: "+ str(total_num_forms))
       usr_grp_forms_cls = formset_factory(form=self.form_class, formset=GenericUserGroupFormset,
                               extra=total_num_forms, max_num=total_num_forms)
       usr_grp_forms_obj = usr_grp_forms_cls(action_type=ExtendedBaseFormSet.ACTION_TYPE_CREATE)
       context = self.get_context_data(formset=usr_grp_forms_obj, total_num_forms=total_num_forms)
       return self.render_to_response(context)

   def post(self, request, *args, **kwargs):
       """
       override ProcessFormView.post() and BaseCreateView.post(), in order to create
       formset to validate and save a list of model forms at once.
       """
       self.object = None
       total_num_forms, error_response = get_total_num_forms_or_400(self, request.POST.get('form-'+ TOTAL_FORM_COUNT, -1))
       if error_response:
           return error_response
       usr_grp_forms_cls = formset_factory(form=self.form_class, formset=GenericUserGroupFormset,
                               extra=total_num_forms, max_num=total_num_forms)
       usr_grp_forms_obj = usr_grp_forms_cls(action_type=ExtendedBaseFormSet.ACTION_TYPE_CREATE, data=request.POST)
       if  usr_grp_forms_obj.is_valid():
           self.success_url = "{protocol}://{hostname}/{path}".format(protocol=request.scheme, hostname=request._get_raw_host(),
                       path=self.success_url_resource )
           return self.formset_valid(formset=usr_grp_forms_obj)
       else:
           return self.formset_invalid(formset=usr_grp_forms_obj, total_num_forms=total_num_forms)


   def formset_invalid(self, **kwargs):
       """ Expand keyword arguments from its superclass, everything else is the same """
       context = self.get_context_data(**kwargs)
       return self.render_to_response(context)

   def formset_valid(self, formset):
       """
       This method runs after the entire formset is validated without any error, it saves each
       form of the formset, then redirect to success URL page specified in this create view.
       """
       formset.insert()
       # only return model object of the first form, self.object is used only when rendering 
       # self.success_url, which can be string with many placefolders.
       # In this view, after succeeding to create new groups, the server should redirect users back
       # to dashboard page (DashBoardView), which is usually static URL string (without any variable
       # placeholder). no need to use self.object in such case.
       self.object = formset.forms[0].instance
       # call grandparent method (FormMixin) instead of parent method (ModelFormMixin), because
       # ModelFormMixin.form_valid() only considers single-form case -- it only saves single form
       # then immediately redirect to success URL page.
       return  FormMixin.form_valid(self, None)


class UserGroupsDeleteView(View):
    success_url_resource  = UserMgtCfg.success_url[__qualname__]

    def post(self, request, *args, **kwargs):
        IDs = request.POST['id_seq'].split(',')
        IDs = [int(i) for i in IDs if i]
        print("IDs : "+ str(IDs))
        qset = GenericUserGroup.objects.filter(id__in=IDs)
        with transaction.atomic():
            qset.delete()
        success_url = "{protocol}://{hostname}/{path}".format(protocol=request.scheme, hostname=request._get_raw_host(),
                        path=self.success_url_resource )
        return HttpResponseRedirect(success_url)


class UserGroupsEditView(UpdateView):
    model = GenericUserGroup
    form_class = GenericUserGroupForm
    template_name = UserMgtCfg.template_name[__qualname__]
    success_url_resource = UserMgtCfg.success_url[__qualname__]
    _400_template_name   = UserMgtCfg.template_name['Http400BadRequestView']

    def get(self, request, *args, **kwargs):
        self.object = None ### self.get_object(queryset=edit_grps)
        IDs = [int(i) for i in kwargs['IDs'].split('/') if i] # skip non-digit character
        total_num_forms, error_response = get_total_num_forms_or_400(self, len(IDs))
        if error_response:
            return error_response
        usr_grp_forms_cls = formset_factory(form=self.form_class, formset=GenericUserGroupFormset,
                                extra=total_num_forms, max_num=total_num_forms)
        usr_grp_forms_obj = usr_grp_forms_cls(action_type=ExtendedBaseFormSet.ACTION_TYPE_EDIT, form_kwargs={"IDs": IDs})
        context = self.get_context_data(formset=usr_grp_forms_obj, total_num_forms=total_num_forms)
        return self.render_to_response(context)

    def post(self, request, *args, **kwargs):
        self.object = None
        IDs = [int(i) for i in kwargs['IDs'].split('/') if i] # skip non-digit character
        total_num_forms = len(IDs)
        total_num_forms2, error_response = get_total_num_forms_or_400(self, request.POST.get('form-'+ TOTAL_FORM_COUNT, -1))
        if error_response or (total_num_forms != total_num_forms2):
            return error_response
        usr_grp_forms_cls = formset_factory(form=self.form_class, formset=GenericUserGroupFormset,
                                extra=total_num_forms, max_num=total_num_forms)
        usr_grp_forms_obj = usr_grp_forms_cls(action_type=ExtendedBaseFormSet.ACTION_TYPE_EDIT, data=request.POST, form_kwargs={"IDs": IDs})
        if  usr_grp_forms_obj.is_valid():
            self.success_url = "{protocol}://{hostname}/{path}".format(protocol=request.scheme, hostname=request._get_raw_host(),
                            path=self.success_url_resource )
            return self.formset_valid(formset=usr_grp_forms_obj)
        else:
            return self.formset_invalid(formset=usr_grp_forms_obj, total_num_forms=total_num_forms)

    def formset_invalid(self, **kwargs):
        context = self.get_context_data(**kwargs)
        return self.render_to_response(context)

    def formset_valid(self, formset):
        formset.update()
        self.object = formset.forms[0].instance
        return  FormMixin.form_valid(self, None)



        # note that both FormMixin and MultipleObjectMixin override get_context_data() from ContextMixin
        # which means get_context_data will be called twice. When calling get_context_data in FormMixin,
        # it would instantiate a new form class if there's no key "form" in its kwargs, which would cause
        # problem in my implementation

# TODO: reconsider to implement mixin for formset, instead of FormMixin for single-form use case
class UsersCreateView(ContextMixin, TemplateResponseMixin, View):
    form_prefix = "user_profile_form"
    template_name = UserMgtCfg.template_name[__qualname__]
    success_url_resource = UserMgtCfg.success_url[__qualname__]
    _400_template_name   = UserMgtCfg.template_name['Http400BadRequestView']
    NON_FORM_ERRORS = 'non_form_errors'

    def get(self, request, *args, **kwargs):
        formparams = {'max_num_form': MAX_NUM_FORM, 'top_level_prefix':self.form_prefix, 'non_field_errors':api_settings.NON_FIELD_ERRORS_KEY,
                'csrf_token': {'name': 'csrfmiddlewaretoken', 'value': csrf.get_token(request)}, self.NON_FORM_ERRORS: self.NON_FORM_ERRORS,
                'subform_prefix': {'emails':EmailAddressForm.prefix, 'phones':PhoneNumberForm.prefix, 'locations':LocationForm.prefix,}}
        context = ContextMixin.get_context_data(self, formparams=formparams)
        return TemplateResponseMixin.render_to_response(self, context)

    def post(self, request, *args, **kwargs):
        """ request content type : multiform, response content type: json """
        http_status = HttpResponseBase.status_code
        response_data = {}
        tot_n_form_field_name = "-".join([self.form_prefix, TOTAL_FORM_COUNT])
        total_num_forms, error_response = get_total_num_forms_or_400(self, request.POST.get(tot_n_form_field_name, -1))
        if error_response:
            http_status = error_response.status_code
            response_data['errors'] = [{self.NON_FORM_ERRORS: ['exceed max. # forms that can be processed at once']}]
        else:
            forms_cls = modelformset_factory(GenericUserProfile, form=GenericUserProfileForm, extra=total_num_forms,
                            max_num=total_num_forms, min_num=total_num_forms, formset=ExtendedBaseModelFormSet)
            formset = forms_cls(data=request.POST, prefix=self.form_prefix, queryset=GenericUserProfile.objects.none())
            if  formset.is_valid():
                print("[to-do] form validation passed, store something.")
                hostname = "{protocol}://{hostname}".format(protocol=request.scheme, hostname=request._get_raw_host())
                res_path = UserMgtCfg.api_url[AccountActivationView.__name__]
                res_path = [USERMGT_APP_PATH] + res_path.split('/')
                res_path = "/".join(res_path[:-1])
                formset.postprocess_if_valid(res_path=res_path, hostname=hostname)
                redirect_path = [hostname] + self.success_url_resource.split('/')
                response_data['success_url'] = "/".join(redirect_path)
            else :
                response_data['errors'] = formset.errors
        return JsonResponse(data=response_data, status=http_status)
        ####     print("total_num_forms ? : "+ str(total_num_forms))
        ####     print("formset.min_num : "+ str(formset.min_num))
        ####     print("formset.initial_form_count() : "+ str(formset.initial_form_count()))
        ####     print("formset.total_form_count() : "+ str(formset.total_form_count()))
        #### print("response_data : "+ str(response_data))




