from django.forms    import  Form, ModelForm
from django.core     import  validators
from django.forms.fields    import IntegerField, FloatField, DecimalField
from django.http.response   import HttpResponseBadRequest
from django.forms.utils     import ErrorList, ErrorDict
from django.core.exceptions import NON_FIELD_ERRORS, ValidationError

DEFAULT_NUM_FORM = 3
MAX_NUM_FORM = 10


def get_total_num_forms_or_400(view_obj, total_num_forms):
    """
    if total_num_forms is NOT found, or greater than max. value, then the server
    should response with 400 Bad Request
    """
    error_response = None
    try:
        if not isinstance(total_num_forms, int):
            total_num_forms = int(total_num_forms)
        if (total_num_forms > MAX_NUM_FORM) or (total_num_forms < 0):
            err_msg = "".join(["total number of edit forms in one page is ", str(total_num_forms) ,\
                    ", which should NOT be greater than ", str(MAX_NUM_FORM), " or less than 0."])
            raise ValueError(err_msg)
    except ValueError as err:
        context = {'return_url': '', 'return_text':'Go Back & Add user group again', 'error':err}
        view_obj.template_name = view_obj._400_template_name
        error_response = view_obj.render_to_response(context)
        error_response.status_code = HttpResponseBadRequest.status_code
    return total_num_forms, error_response


class ErrorHandleMixin:
    def copy_errors(self, dst):
        """
        copy errors from this form to destination form, the common use case would be
        to collect error(s) from each of subforms to the parent form at each level.
        """
        #### print("copy_errors at form level "+ self.prefix +" error : "+ str(self._errors))
        if self._errors in validators.EMPTY_VALUES:
            return
        local_prefix = self.prefix.split('-')[-1]
        tmp = ErrorDict()
        for k, v in self._errors.items():
            tmp[k] = v
            #### ValidationError("some_message")
            # directly modify self._error instead of calling self.add_error(), TODO: find better way
        if (not tmp in validators.EMPTY_VALUES):
            if dst.get(local_prefix, None):
                dst[local_prefix].update(tmp)
            else:
                dst[local_prefix] = tmp


class FormValidationMixin:
    def postprocess_if_valid(self, **kwargs):
        """
        subclasses may override this function for extra post processing code
        after form validation passes
        """
        pass


class ExtendedBaseForm(Form, ErrorHandleMixin, FormValidationMixin):
    pass


class ExtendedModelForm(ModelForm, ErrorHandleMixin, FormValidationMixin):
    def save(self, commit=True, **kwargs):
        """ override ModelForm.save(), to provide more arguments to ORM layer """
        if self.errors:
            raise ValueError(
                "The %s could not be %s because the data didn't validate." % (
                    self.instance._meta.object_name,
                    'created' if self.instance._state.adding else 'changed',
                )
            )
        if commit:
            # If committing, save the instance and the m2m data immediately.
            self.instance.save(**kwargs)
            self._save_m2m()
        else:
            # If not committing, add a method to the form to allow deferred
            # saving of m2m data.
            self.save_m2m = self._save_m2m
        return self.instance
