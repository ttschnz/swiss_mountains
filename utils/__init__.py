from . import box

def url_to_ref(url:str)->str:
    return url.split('/')[-1].split('.')[0]
