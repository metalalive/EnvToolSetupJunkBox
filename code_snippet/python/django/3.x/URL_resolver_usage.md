
### [reverse()](https://docs.djangoproject.com/en/dev/ref/urlresolvers/#reverse)
For hierarchical path , you have to specify namespace in reverse() function and `name` argument in path() in `urls.py` , for example :

* in `your_app/urls/__init__.py`
```python
from django.urls  import  path, re_path, include
urlpatterns = [
    path('product/', include('your_app.urls.productmgt')),
]
```

* in `your_app/urls/productmgt.py`
```
from django.urls import  path
from your_app.views import ProductAttrTypeProxyView, FabricationIngredientProxyView
app_name = 'productmgt'
urlpatterns = [
    path('attrtypes',              ProductAttrTypeProxyView.as_view() ,name='ProductAttrTypeProxyView' ),
    path('ingredients',                 FabricationIngredientProxyView.as_view() ,name='FabricationIngredientProxyView0' ),
    path('ingredient/<slug:ingre_id>',  FabricationIngredientProxyView.as_view() ,name='FabricationIngredientProxyView1' ),
]
```

* in python shell console
```python
from django.urls import reverse
# `namespace` comes from `app_name` in urls/productmgt.py
reverse('productmgt:ProductAttrTypeProxyView')
> '/product/attrtypes'
reverse('productmgt:FabricationIngredientProxyView0')
> '/product/ingredients'
reverse('productmgt:FabricationIngredientProxyView1', kwargs={'ingre_id':198})
> '/product/ingredient/198'
```
Note that `reverse()` seems to rely on `name` argument in `django.urls.path()` which is NOT really useful ...


### [URLResolver](https://www.fullstackpython.com/django-urls-urlresolver-examples.html)
`django.urls.resolvers.URLResolver` makes hierarchical path resolution possible for large-scale applications
For example, if you have  `your_app/urls/__init__.py` which looks like this :
```python
from django.urls  import  path, re_path, include
urlpatterns = [
    ...
    path('product/', include('your_app.urls.productmgt')),
    ...
]
```

* Basic usage
```python
from django.urls.resolvers import URLResolver
from your_app.urls import urlpatterns
resolvers = filter(lambda path: isinstance(path, URLResolver), urlpatterns)
resolvers = list(resolvers)
resolvers[0]
> <URLResolver <module 'your_app.urls.productmgt' from '/PATH/TO/your_app/urls/productmgt.py'> (productmgt:productmgt) 'product/'>
```

`URLResolver` provides instance method `reverse()`, which works pretty much as the same as [`django.url.reverse`](./URL_resolver_usage.md#reverse()), to resolve **partial valid URL path** (omit namespace) from given input string :

```python
resolvers[0].reverse('FabricationIngredientProxyView0')
> 'ingredients'
```


#### [iterate all valid URL patterns](https://stackoverflow.com/a/1275601/9853105)
```
from django.urls.resolvers import URLResolver
from your_app.urls import urlpatterns
resolvers = filter(lambda path: isinstance(path, URLResolver), urlpatterns)
resolvers = list(resolvers)
set(v[1] for k,v in resolvers[0].reverse_dict.items())
> {'attrtypes$', 'ingredient/(?P<ingre_id>[-a-zA-Z0-9_]+)$',)
```
