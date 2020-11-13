from django.core            import  validators
from django.db              import  models
from django.forms           import  BaseFormSet, ChoiceField
from django.forms.models    import  BaseModelFormSet
from django.forms.utils     import  ErrorDict, ErrorList
from django.core.exceptions import  NON_FIELD_ERRORS, ValidationError

from .validators  import  ClosureSingleTreeLoopValidator, TreeNodesLoopValidator, ClosureCrossTreesLoopValidator

trace_old_paths = {}
def trace_duplicate_old_path(obj):
    try:
        record = trace_old_paths[obj.id]
        raise ValueError("duplicate records found, ID in closure table :"+ str(obj))
    except KeyError:
        trace_old_paths[obj.id] = obj

def clear_trace_old_path():
     trace_old_paths.clear()



class FormsetValidatorMixin:

    def run_validators(self):
        """
        There's no place in Django's default BaseFormSet class to validate a formset by checking
        several fields of different forms at once, this function dedicates such case.
        """
        if hasattr(self, '_validators'):
            for vfn, values in self._validators:
                try:
                    vfn(**values)
                except ValidationError as e:
                    _dict = self._errors[-1].get(NON_FIELD_ERRORS, None)
                    if _dict is None:
                        self._errors.append(ErrorDict())
                    _list = self._errors[-1].get(NON_FIELD_ERRORS, None)
                    if _list is None:
                        self._errors[-1][NON_FIELD_ERRORS] = ErrorList()
                    self._errors[-1][NON_FIELD_ERRORS].append(e.message)
                    #### self._errors.append({NON_FIELD_ERRORS: e.message})
            #print("".join(["self.has_error()  :  ", str(self._errors), " ....", str(self.has_error())]))
        return not self.has_error()


    def add_validator(self, vfn_cls, init_kwargs: dict, call_kwargs: dict):
        """ formset-wide validator and a set of arguments to validate """
        if vfn_cls:
            init_kwargs = init_kwargs or {}
            vfn = vfn_cls(**init_kwargs)
            if not hasattr(self, '_validators'):
                self._validators = []
            self._validators.append(tuple([vfn, call_kwargs]))


    def has_error(self):
        """ indicate whether there's any formset-wide validation error """
        out = False
        if self._errors is None:
            out = False
        elif self._errors in validators.EMPTY_VALUES:
            out = False
        else:
            for e in self._errors:
                if not e in validators.EMPTY_VALUES:
                    out = True
        return out

    def prepare_formset_wide_validators(self):
        """
        Subclass should override this function to provide application-specific
        formset-wide validator
        """
        pass

    def postprocess_if_valid(self, **kwargs):
        """
        post-processing code after the entire formset validation passes .
        """
        for form in self.forms:
            form.postprocess_if_valid(**kwargs)



class ErrorHandleMixin:
    def copy_errors(self, dst):
        """
        copy errors of current formset to given destination form
        """
        #### print("copy_errors at formset level "+ self.prefix +" , self._errors : "+ str(self._errors))
        if self._errors in validators.EMPTY_VALUES:
            return
        local_prefix = self.prefix.split('-')[-1]
        ## print("extended model form set, local_prefix : "+ local_prefix)
        tmp = ErrorDict()
        for form in self.forms:
            form.copy_errors(dst=tmp)
        # copy form-wide / formset-wide error
        tmp2 = ErrorList()
        for k,v in self._errors[-1].items(): # TODO: is it the only way to check whether non-field error exists ?
            if k is NON_FIELD_ERRORS:
                tmp2.append(v)
        if (not tmp in validators.EMPTY_VALUES) or (not tmp2 in validators.EMPTY_VALUES):
            if dst.get(local_prefix, None):
                dst[local_prefix].update(tmp)
            else:
                dst[local_prefix] = tmp
        if (not tmp2 in validators.EMPTY_VALUES):
            if dst[local_prefix].get(NON_FIELD_ERRORS, None):
                dst[local_prefix][NON_FIELD_ERRORS].update(tmp2)
            else:
                dst[local_prefix][NON_FIELD_ERRORS] = tmp2



class ExtendedBaseFormSet(BaseFormSet, FormsetValidatorMixin, ErrorHandleMixin):

    ACTION_TYPE_CREATE = 1
    ACTION_TYPE_EDIT   = 2
    ACTION_TYPE_DELETE = 3
    valid_action_types = [ACTION_TYPE_CREATE, ACTION_TYPE_EDIT, ACTION_TYPE_DELETE]

    def __init__(self, action_type, data=None, files=None, auto_id='id_%s', prefix=None,
                 initial=None, error_class=ErrorList, form_kwargs=None):
        if not action_type in self.valid_action_types:
            err_msg = [str(action_type) ," is NOT valid action type"]
            raise ValueError("".join(err_msg))
        self.action_type = action_type
        # NOTE: must set initial value to some form fields, otherwise formset validation 
        #       to text field will go wrong ( Django bug ?)
        initial = [] if initial is None else initial
        for idx in range(self.max_num):
            try:
                initial[idx]
            except IndexError:
                initial.append({})
            initial[idx]['name'] = ''
        self._init_with_IDs(form_kwargs=form_kwargs, initial=initial)
        super().__init__(data=data, files=files, auto_id=auto_id, prefix=prefix,
                 initial=initial, error_class=error_class, form_kwargs=form_kwargs)


    def _init_with_IDs(self, initial, form_kwargs):
        if isinstance(form_kwargs, dict):
            IDs = form_kwargs.get('IDs')
            if IDs:
                id_union = models.Q(id__in=IDs)
                model_cls = self.form._meta.model # TODO: use select_related() and prefetch_related()
                model_qset = model_cls.objects.filter(id_union).order_by(model_cls.id.field.name)
                form_kwargs['instance'] = list(model_qset)
                self.init_IDs_extension(model_qset, IDs, initial)
                # if idx is not equal to length of ID list, chances are that the ID list specified
                # by client cannot be found at model layer
                num_empty_init_params = len(IDs) - len(model_qset)
                for idx in range(num_empty_init_params):
                    initial.remove(initial[-1])
                self.extra   -= num_empty_init_params
                self.max_num -= num_empty_init_params
                # print("initial : "+ str(initial))
                # print("self.max_num : "+ str(self.max_num))
                # print("IDs : "+ str(IDs))
                del form_kwargs['IDs']


    def init_IDs_extension(self, model_qset, IDs, initial):
        """
        Subclasses must override this function for adding custom initial parameters to
        the formset.
        """
        pass


    def get_form_kwargs(self, index):
        return {k: v[index] for k,v in self.form_kwargs.items()}


    def is_valid(self):
        """
        This application supports bulk CURD operations in a single view,
        The formset data received from client could go wrong , here is where
        you can do extra formset-wide validation.
        """
        forms_valid =  super(ExtendedBaseFormSet, self).is_valid()
        if forms_valid:
            self.prepare_formset_wide_validators()
            forms_valid = self.run_validators()
        # return forms_valid and not self.non_form_errors()
        return forms_valid


    def _prepare_distinct_IDs_validators(self):
        """ ID of each form in a formset must be distinct """
        init_kwargs = None
        if self.action_type is ExtendedBaseFormSet.ACTION_TYPE_EDIT:
            IDs = [form.cleaned_data['id'] for form in self.forms]
            call_kwargs = {'num_list': IDs}
            from .validators  import  UniqueListValueValidator
            validator_cls = UniqueListValueValidator
        else:
            call_kwargs = None
            validator_cls = None
        return validator_cls, init_kwargs, call_kwargs
#### end of ExtendedBaseFormSet



class EditTreeNode(object):
    """
    Internal class to represent edit tree node, when updating to closure table.
    """
    def __init__(self, model_obj, depth):
        self.obj = model_obj
        self.depth = depth


class ClosureTableFormsetMixin:
    """ generic class for maintaining closure table """

    def init_parent_IDs(self, model_qset, IDs, initial, closure_model):
        """ Retrieve parent ID (if exists) for each ID in the given ID list """
        parents = closure_model.get_parents(IDs=IDs, depth_min=1, depth_max=1)
        idx = 0
        for grp in model_qset:
            parent = parents.get(grp.id)
            if parent:  # It is normal that some node (e.g. root node) don't have parent ID
                initial[idx]['exist_parent'] = parent[ closure_model.ancestor.field.name ]
            else:
                initial[idx]['exist_parent'] = ''
            idx += 1
        # print("edit_grp_parents : "+ str(edit_grp_parents))
        del parents


    def _prepare_parent_validator(self, root_id, model_cls):
        """ prepare loop validator for each edit tree """
        validator = ClosureSingleTreeLoopValidator(
                         T2_root_id=root_id,  closure_model=model_cls,
                         ancestor_column_name   = model_cls.ancestor.field.name,
                         descendant_column_name = model_cls.descendant.field.name,
                    )
        return validator


    def _add_existing_parents_field(self, form, validators_list):
        validators_list = validators_list or []
        choices  = []
        if form.is_bound:
            field_name = "".join([form.prefix, '-', 'exist_parent'])
            exist_parent = self.data.get(field_name)
            choices.append(tuple([exist_parent, 'xxx']))
        elif form.initial.get('exist_parent', None):
            exist_parent = form.initial['exist_parent']
            choices.append(tuple([exist_parent, 'xxx']))
        form.fields["exist_parent"] = ChoiceField(choices=tuple(choices), required=False, validators=tuple(validators_list))


    def _add_new_parents_field(self, form):
        total_num_forms = self.total_form_count()
        new_grps = []
        for n in range(total_num_forms):
            label = "-".join([self.prefix, str(n)])
            if label != form.prefix:
                new_grps.append(tuple([n, label]))
        #### print("new_grps : "+ str(new_grps))
        choices = ((validators.EMPTY_VALUES[1], '-------'), ) # empty string as default choice
        choices = choices + tuple(new_grps)
        form.fields["new_parent"] = ChoiceField(choices=choices, required=False,)


    def prepare_cycle_detection_validators(self, forms, model_cls):
        """
        This mixin class maintains trees in a closure table, which should be acyclic
        . This function provides validator class, arranges root of each edit tree
        and its new parent to a set of edges e.g. (new_parent_id, root_id), feed
        the edges to the cycle-detection validator.
        """
        tree_edge = []
        init_kwargs = {}
        call_kwargs = {}
        init_kwargs['tree_edge']  = tree_edge
        for idx in range(len(forms)):
            form = forms[idx]
            exist_parent_id = form.cleaned_data['exist_parent']
            if self.action_type is ExtendedBaseFormSet.ACTION_TYPE_CREATE:
                new_parent_id = form.cleaned_data['new_parent']
                parent_id = 0
                if (exist_parent_id in validators.EMPTY_VALUES) and (not new_parent_id in validators.EMPTY_VALUES):
                    parent_id = int(new_parent_id)
                else:
                    parent_id = TreeNodesLoopValidator.ROOT_OF_TREE # which means root node
                tree_edge.append(tuple([parent_id, idx]))
            elif self.action_type is ExtendedBaseFormSet.ACTION_TYPE_EDIT:
                if (exist_parent_id in validators.EMPTY_VALUES):
                    exist_parent_id = TreeNodesLoopValidator.ROOT_OF_TREE # which means root node
                parent_id = int(exist_parent_id)
                tree_edge.append(tuple([parent_id, form.cleaned_data['id']]))
        if self.action_type is ExtendedBaseFormSet.ACTION_TYPE_CREATE:
            validator_cls = TreeNodesLoopValidator
        elif self.action_type is ExtendedBaseFormSet.ACTION_TYPE_EDIT:
            init_kwargs['closure_model']          = model_cls
            init_kwargs['depth_column_name']      = model_cls.depth.field.name
            init_kwargs["ancestor_column_name"]   = model_cls.ancestor.field.name
            init_kwargs["descendant_column_name"] = model_cls.descendant.field.name
            validator_cls = ClosureCrossTreesLoopValidator
        else:
            validator_cls = None
        return validator_cls, init_kwargs, call_kwargs


    def get_sorted_insertion_forms(self):
        """ reorder the forms in case there are dependencies among the newly added forms """
        if hasattr(self, '_sorted_insertion_forms'):
            return self._sorted_insertion_forms
        insert_after = {}
        unsorted_forms = []
        sorted_forms = []
        for form in self.forms:
            exist_parent_id = form.cleaned_data['exist_parent']
            new_parent_id = form.cleaned_data['new_parent']
            if exist_parent_id in validators.EMPTY_VALUES:
                if new_parent_id in validators.EMPTY_VALUES:
                    sorted_forms.append(form)
                else: # record the position the current form should be in sorted list
                    unsorted_forms.append(form)
                    insert_after[form.prefix] = self.forms[int(new_parent_id)]
            else:
                sorted_forms.append(form)
        # print("insert_after : "+ str(insert_after))
        while len(sorted_forms) < len(self.forms) and not unsorted_forms in validators.EMPTY_VALUES:
            for form in unsorted_forms:
                try:
                    new_parent_id = sorted_forms.index(insert_after[form.prefix])
                    form.cleaned_data['new_parent'] = new_parent_id
                    sorted_forms.append(form)
                    unsorted_forms.remove(form)
                except ValueError:
                    pass # skip if parent form required hasn't been added to the sorted list.
        self._sorted_insertion_forms = sorted_forms
        return sorted_forms


    def get_insertion_parent_id(self, form):
        sorted_forms = self.get_sorted_insertion_forms()
        # get new parent ID from each form
        exist_parent_id = form.cleaned_data['exist_parent']
        new_parent_id = form.cleaned_data['new_parent']
        parent_id = 0
        if exist_parent_id in validators.EMPTY_VALUES:
            if new_parent_id in validators.EMPTY_VALUES:
                parent_id = -1
            else:
                parent_id = sorted_forms[int(new_parent_id)].instance.id
                #print(str(form.prefix) +", id : "+ str(form.instance.id) + ", new_parent_id : "+ str(new_parent_id) +", parent id: "+ str(parent_id))
        else:
            parent_id = int(exist_parent_id)
        return parent_id
    #### end of get_insertion_parent_id()


    def _init_edit_tree(self, forms, model_cls):
        """
        This is part of internal functions that maintain closure table structure.
        This function initializes data structure for a set of edit trees, stored in the form data
        on client (POST) request. If the form data contains a tree whose parent ID will be changed,
        the change of the tree will be recorded to the output edit tree structure.
        """
        IDs = [form.instance.id for form in forms]
        old_parents = model_cls.get_parents(IDs=IDs, depth_min=1, depth_max=1)
        new_parents = {form.instance.id: form for form in forms}
        for k,v in new_parents.items():
            pid = v.cleaned_data['exist_parent']
            pid = TreeNodesLoopValidator.ROOT_OF_TREE if pid in validators.EMPTY_VALUES else int(pid)
            v.cleaned_data['exist_parent'] = pid
        edit_trees = {}
        # check if a tree will change its position in the entire closure table
        for k,v in old_parents.items():
            new_parent_ID = new_parents[k].cleaned_data['exist_parent']
            old_parent_ID = v[model_cls.ancestor.field.name]
            if new_parent_ID != old_parent_ID:
                edit_trees[k] = {"old": {"parent_id":old_parent_ID, }, "new": {"parent_id":new_parent_ID, }, "form":new_parents[k] }
                new_parents[k].edit_tree = edit_trees[k]
        for k,v in new_parents.items():
            new_parent_ID = v.cleaned_data['exist_parent']
            try:
                old_parent_ID = old_parents[k][model_cls.ancestor.field.name]
            except KeyError as err:
                old_parent_ID = TreeNodesLoopValidator.ROOT_OF_TREE
            if new_parent_ID != old_parent_ID and not edit_trees.get(k, None):
                edit_trees[k] = {"old": {"parent_id":old_parent_ID,}, "new": {"parent_id":new_parent_ID,}, "form":v}
                v.edit_tree = edit_trees[k]
        print("".join(["old_parents : ", str(old_parents) ]))
        print("".join(["new_parents : ", str({k:v.cleaned_data['exist_parent'] for k,v in new_parents.items()}), ]))
        del old_parents
        del new_parents
        return edit_trees


    def _reorder_edit_tree(self, edit_trees_in, model_cls):
        """
        This is part of internal functions that maintain closure table structure.
        The given edit tree list has to be reordered if there are dependencies found
        among the trees, (e.g. tree A, B in the edit tree list, new parent of A is one
        of descendant of B) reorder ensures correctness of update sequence.
        """
        update_after = {}
        unsorted_trees = {}
        sorted_trees = {}
        # check if new parent of each tree is a descendant of another tree in the edit list,
        # if so, then it is essential to build  dependancy graph
        desc_fname = model_cls.descendant.field.name
        anc_fname  = model_cls.ancestor.field.name
        condition = models.Q()
        for k,v in edit_trees_in.items():
            id_list = [k2 for k2 in edit_trees_in.keys()]
            id_list.remove(k)
            condition |= (models.Q(descendant=v['new']['parent_id']) & models.Q(ancestor__in=id_list))
        qset = model_cls.objects.select_related(anc_fname, desc_fname).filter(condition).order_by(model_cls.depth.field.name)
        for k,v in edit_trees_in.items(): # estimate update dependency
            for q in qset: # TODO: figure out why this cannot be outer loop
                if q.descendant.id == v['new']['parent_id']:
                    if not v.get('dependency'):
                        v['dependency'] = q.ancestor.id
                        unsorted_trees[k] = v
                    if not update_after.get(k):
                        update_after[k] = []
                    update_after[k].append(q.ancestor.id)
                    break
        for k,v in edit_trees_in.items():
            if not v.get('dependency'): # which means no update dependency prior to this tree
                 v['dependency'] = TreeNodesLoopValidator.ROOT_OF_TREE
                 sorted_trees[k] = v
        remove = []
        while len(sorted_trees) < len(edit_trees_in): # start reordering
            print("".join(["update_after : ", str(update_after), ]))
            for k,v in unsorted_trees.items():
                sorted_id_list = [k2 for k2 in sorted_trees.keys()]
                diff = set(update_after[k]) - set(sorted_id_list)
                update_after[k] = list(diff)
                if update_after[k] in validators.EMPTY_VALUES:
                    sorted_trees[k] = v
                    remove.append(k)
            for k in remove:
                del unsorted_trees[k]
                del update_after[k]
            remove.clear()
        print("".join(["order dependancies : ", str({k:v['dependency'] for k,v in edit_trees_in.items()}), ]))
        return sorted_trees


    def _construct_edit_ancestors(self, edit_trees_in, model_cls):
        """
        This is part of internal functions that maintain closure table structure.
        This function estimates difference of all ancestors before/after editing each tree in the edit list.
        [I/O]
        Input : edit tree list, with the attribute "old/new parent ID", "dependency" for each edit tree
        Output: insert new attribute "old/new ancestors", to describe the ancestors and the depths.
        """
        desc_fname = model_cls.descendant.field.name
        anc_fname  = model_cls.ancestor.field.name
        descendant_asc_order = "".join(['', desc_fname])
        depth_desc_order = "".join(['-', model_cls.depth.field.name])
        condition = models.Q(descendant__in=[v["old"]["parent_id"] for v in edit_trees_in.values()])
        qset_old_ascs = model_cls.objects.select_related(anc_fname, desc_fname).filter(condition
                        ).order_by(descendant_asc_order, depth_desc_order)
        condition = models.Q(descendant__in=[v["new"]["parent_id"] for v in edit_trees_in.values()])
        qset_new_ascs = model_cls.objects.select_related(anc_fname, desc_fname).filter(condition
                        ).order_by(descendant_asc_order, depth_desc_order)
        qset_old_ascs = list(qset_old_ascs)
        qset_new_ascs = list(qset_new_ascs)
        for k, v in edit_trees_in.items():
            v['old']['ancestors'] = []
            v['new']['ancestors'] = []
            for a in qset_old_ascs: # retrieve old ancestors of each edit tree from model layer
                if a.descendant.id == v["old"]["parent_id"]:
                    v['old']['ancestors'].append(EditTreeNode(model_obj=a.ancestor, depth=a.depth))
            if v['dependency'] == TreeNodesLoopValidator.ROOT_OF_TREE:
                for a in qset_new_ascs:
                    if a.descendant.id == v["new"]["parent_id"]:
                        v['new']['ancestors'].append(EditTreeNode(model_obj=a.ancestor, depth=a.depth))
        while True: # construct new ancestors of each edit tree (also consider update dependency)
            num_ancestors_added = 0
            for k, v in edit_trees_in.items():
                if v['dependency'] != TreeNodesLoopValidator.ROOT_OF_TREE:
                    if v['new']['ancestors'] in validators.EMPTY_VALUES:
                        dependency_ancestors = edit_trees_in[v['dependency']]['new']['ancestors']
                        v['new']['ancestors'] = [EditTreeNode(model_obj=a.obj, depth=a.depth) for a in dependency_ancestors]
                        copy_flag = False
                        for a in qset_new_ascs:
                            if a.ancestor.id == v['dependency'] and a.descendant.id == v["new"]["parent_id"]:
                                copy_flag = True
                            if copy_flag:
                                v['new']['ancestors'].append(EditTreeNode(model_obj=a.ancestor, depth=a.depth))
                            if a.ancestor.id == v["new"]["parent_id"]:
                                copy_flag = False
                        num_ancestors_added += len(v['new']['ancestors'])
            if num_ancestors_added == 0:
                break
        for k, v in edit_trees_in.items(): # update depth of new ancestors
            ancestor_len = len(v['new']['ancestors'])
            for idx in range(ancestor_len):
                v['new']['ancestors'][idx].depth = ancestor_len - idx - 1
        qset_old_ascs.clear()
        qset_new_ascs.clear()


    def _construct_edit_descendants(self, edit_trees_in, model_cls):
        """
        This is part of internal functions that maintain closure table structure.
        This function estimates difference of all descendants before/after editing each tree in the edit list.
        [I/O]
        Input : edit tree list, with the attribute "new ancestors", "dependency" for each edit tree
        Output: insert new attribute "old/new descendants", to describe the descendants and the depths.
        """
        desc_fname = model_cls.descendant.field.name
        anc_fname  = model_cls.ancestor.field.name
        qset = list(model_cls.objects.select_related(anc_fname, desc_fname).filter(ancestor__in=list(edit_trees_in.keys())))
        for k, v in edit_trees_in.items(): # retrieve old descendants of each edit tree from model layer
            v['old']['descendants'] = []
            v['old']['moving_subtree_root'] = []
            v['new']['moving_subtree_root'] = []
            for d in qset:
                if d.ancestor.id == k:
                    v['old']['descendants'].append(EditTreeNode(model_obj=d.descendant, depth=d.depth))
        for k, v in edit_trees_in.items(): # find subtrees in old descendants, the subtree is also another tree in edit list
            old_descs = [d.obj.id for d in v['old']['descendants']]
            for k2 in edit_trees_in.keys():
                if k != k2 and k2 in old_descs:
                    v['old']['moving_subtree_root'].append(k2)
        # find subtrees that will be (1) added to current edit tree (from another edit tree)
        # (2) still under the same tree, but in different position.
        for k, v in edit_trees_in.items():
            if v['dependency'] != TreeNodesLoopValidator.ROOT_OF_TREE:
                edit_trees_in[v['dependency']]['new']['moving_subtree_root'].append(k)
        for k, v in edit_trees_in.items(): # find the subtrees that will be moved out (to another edit tree)
            v['new']['move_out_subtree_root'] = []
            for k2 in v['old']['moving_subtree_root']:
                new_ancs = [a.obj.id for a in edit_trees_in[k2]['new']['ancestors']]
                if not k in new_ancs:
                    v['new']['move_out_subtree_root'].append(k2)
        # Get rid of those subtree(s) from each edit tree, if the root of the subtree(s) also appears in edit tree list.
        remove = []
        for k, v in edit_trees_in.items():
            exc_ids = set(v['new']['moving_subtree_root']) | set(v['new']['move_out_subtree_root'])
            for w in exc_ids:
                remove += [d.obj.id for d in edit_trees_in[w]['old']['descendants']]
            v['new']['descendants'] = [d for d in v['old']['descendants'] if not d.obj.id in remove]
            remove.clear()
        for k, v in edit_trees_in.items():
            print("".join([ "edit tree root: ", str(k),
                            ", moving (old): ",    str([d for d in v['old']['moving_subtree_root']] ),
                            ", moving (new): ",    str([d for d in v['new']['moving_subtree_root']] ),
                            ", move-out : ", str([d for d in v['new']['move_out_subtree_root']] ),
                            "",
                ]))
        for k, v in edit_trees_in.items():
            print("".join(["old ancestors : ", str([a.obj.id for a in v['old']['ancestors']])]))
            print("".join(["new ancestors : ", str([a.obj.id for a in v['new']['ancestors']])]))
            print("".join(["old descendants : ", str([d.obj.id for d in v['old']['descendants']])]))
            print("".join(["new descendants : ", str([d.obj.id for d in v['new']['descendants']])]))
            print("".join(["depth of new ancestors : ", str([a.depth  for a in v['new']['ancestors']]), ]))
            print("".join(["depth of new descendants : ", str([d.depth  for d in v['new']['descendants']]), ]))
            print("\n")
        for k, v in edit_trees_in.items():
            v['old']['moving_subtree_root'].clear()
            v['new']['moving_subtree_root'].clear()
            v['new']['move_out_subtree_root'].clear()
            v['old']['descendants'].clear()
            del v['old']['moving_subtree_root']
            del v['new']['moving_subtree_root']
            del v['new']['move_out_subtree_root']
            del v['old']['descendants']
        qset.clear()


    def construct_edit_paths(self, model_cls, edit_tree):
        """
        This is part of internal functions that maintain closure table structure.
        fetch all model instances, that represents paths from old root of each tree to old parent of each tree,
        write the new paths (from new root to new parent of each tree) into the model object instances that
        are previously retrieved.
        [I/O]
        Input : edit tree list, with the attribute "old/new ancestors", "old/new descendants" for each edit tree
        Output: 3 lists of model instances that represents create/update/delete operaitons on the closure table
        """
        #### edit_trees_in = self.edit_trees
        desc_fname = model_cls.descendant.field.name
        anc_fname  = model_cls.ancestor.field.name
        if not hasattr(self, '_delete_path_objs'):
            self._delete_path_objs = []
        create_path_objs = []
        update_path_objs = []
        delete_path_objs = self._delete_path_objs
        #### for v in edit_trees_in.values():
        v = edit_tree
        old_path_len = len(v['old']['ancestors'])
        new_path_len = len(v['new']['ancestors'])
        num_new_descendants = len(v['new']['descendants'])
        desc_id_union = models.Q(descendant__in=[d.obj.id for d in v['new']['descendants']])
        anc_id_union  = models.Q(ancestor__in=  [a.obj.id for a in v['old']['ancestors']])
        old_paths = model_cls.objects.select_related(anc_fname, desc_fname).filter(
                    desc_id_union & anc_id_union).order_by(model_cls.id.field.name)
        idx = jdx = kdx = 0
        for jdx in range(new_path_len):
            for kdx in range(num_new_descendants):
                selected_obj_list = update_path_objs
                idx = jdx * num_new_descendants + kdx
                if idx >= len(old_paths): # if running out of space in old_paths
                    if len(delete_path_objs) > 0: # see if I can reuse the model instances that will be deleted
                        obj = delete_path_objs.pop(0)
                    else: # time to create new model instance object(s)
                        obj = model_cls()
                        selected_obj_list = create_path_objs
                else:
                    obj = old_paths[idx]
                obj.descendant = v['new']['descendants'][kdx].obj
                obj.ancestor = v['new']['ancestors'][jdx].obj
                obj.depth    = v['new']['ancestors'][jdx].depth + 1 + v['new']['descendants'][kdx].depth
                selected_obj_list.append(obj)
        #### print("".join(["idx : ", str(idx), ", jdx : ", str(jdx), ", kdx : ", str(kdx), ]))
        if old_path_len > new_path_len: # put objects that are no longer use to delete list, or they will be used later
            idx = (1 + jdx) * (1 + kdx) if new_path_len > 0 else 0
            while idx < len(old_paths):
                delete_path_objs.append(old_paths[idx])
                idx += 1
        return create_path_objs, update_path_objs


    def get_sorted_update_forms(self, closure_model):
        """
        input : client form data in the formset object instance
        Output: 3 lists of model instances that represents create/update/delete operaitons
                on the closure table
        """
        if not hasattr(self, '_sorted_update_forms'):
            # check whether parent is modified.
            edit_trees = self._init_edit_tree(self.forms, model_cls=closure_model)
            # reorder if there's cascading updates e.g. new parent of one node is in subtree of another node in the update list
            edit_trees = self._reorder_edit_tree(edit_trees, model_cls=closure_model)
            # construct old/new ancestors for each edit tree
            self._construct_edit_ancestors(edit_trees, model_cls=closure_model)
            # construct old/new descendants for each edit tree
            self._construct_edit_descendants(edit_trees, model_cls=closure_model)
            sorted_forms = [v['form'] for v in edit_trees.values()]
            excluded_forms = set(list(self.forms)) - set(sorted_forms)
            self._sorted_update_forms = list(excluded_forms) + sorted_forms
        #### # print("".join(["edit_trees : ", str(edit_trees), "\n"]))
        return self._sorted_update_forms
    #### end of get_sorted_update_forms()


    def clean_deleted_closure_path(self):
        if not hasattr(self, '_delete_path_objs'):
            return
        for m in self._delete_path_objs:
            print("".join(["deleting path id ", str(m.id) ," : ", str(m.ancestor.id), " to ", str(m.descendant.id), " (dep:", str(m.depth) ,")" ]))
            trace_duplicate_old_path(m)
        for m in self._delete_path_objs:
            m.delete(hard=True) # currently no built-in bulk delete function
        self._delete_path_objs.clear()
        clear_trace_old_path()


        # for k,v in edit_trees_in.items():
        #     if qset.count() > 0:
        #         v['dependency'] = qset[0].ancestor.id
        #         unsorted_trees[k] = v
        #         update_after[k] = [k2.ancestor.id for k2 in qset]
        #     else:
        #         v['dependency'] = TreeNodesLoopValidator.ROOT_OF_TREE
        #         sorted_trees[k] = v

        # sort by (1) subtract old_parents and new_parents
        # (2) if one node is subtree of other node in the update list (TODO)
        #### for k, v in edit_trees.items():
        ####     node_id = k
        ####     v["old_parent"]["root_len"] = GenericUserGroupClosure.objects.filter(
        ####             descendant=v["old_parent"]["id"]).count() if v["old_parent"]["id"] > 0 else 0
        ####     v["new_parent"]["root_len"] = GenericUserGroupClosure.objects.filter(
        ####             descendant=v["new_parent"]["id"]).count() if v["new_parent"]["id"] > 0 else 0
        #### edit_trees = {k:v for k,v in sorted(edit_trees.items(),
        ####                     key=lambda item:item[1]["new_parent"]["root_len"] - item[1]["old_parent"]["root_len"])}



class ExtendedBaseModelFormSet(BaseModelFormSet, FormsetValidatorMixin, ErrorHandleMixin):
    # TODO: figure out why this function is called three times
    def clean(self):
        print("ExtendedBaseModelFormSet why call twice ? clean(): "+ self.prefix)
        super().clean()
        if not hasattr(self, 'already_cleaned'):
            setattr(self, 'already_cleaned', True)
        else:
            return
        self.prepare_formset_wide_validators()
        self.run_validators()



