//! An elegant dictionary and JSON handler (Rust implementation of python's lib dictor)
//! 
//! Dictor is a Python JSON and Dictionary (Hash, Map) handler.
//! Dictor takes a dictionary or JSON data and returns value for a specific key.
//! 
//! If Dictor doesnt find a value for a key, or if JSON or Dictionary data is missing the key, the return value is either None or whatever fallback value you provide.
//! 
//!Dictor is polite with Exception errors commonly encountered when parsing large Dictionaries/JSONs.
//!Using Dictor eliminates the repeated use of try/except blocks in your code when dealing with lookups of large JSON structures, as well as providing flexibility for inserting fallback values on missing keys/values.

use std::fmt::Display;

use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyString, PyList, PyBool, PyFloat, PyInt};
use pyo3::{ToPyObject, PyAny, PyErr};
use pyo3::{types::{PyDict, PyModule}, PyResult, pymodule, Python, PyObject, exceptions::PyValueError,
wrap_pyfunction, pyfunction};


const DOT: &str = ".";
const SLASH: &str = "/";


#[derive(Debug)]
pub struct Input{
    args: Vec<String>,
    delimiter: Option<String>
}
impl Input {
    fn new(raw_input: String, delimiter: String) -> Self {
        let args = raw_input.split(&delimiter).map(|s| s.to_owned()).collect();
        Self { args: args, delimiter: Some(delimiter) }
    }


#[derive(Debug)]
pub enum ParseError{
    InvalidDelimiter(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self))
    }
}


impl TryFrom<String> for Input{
    type Error = ParseError;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let escaped_path = r"\.".to_string();
        if value.contains(&escaped_path){
            let mut args: Vec<String> = vec![];
            let vec_value: Vec<&str> = value.split(".").collect();
            let max_pos = vec_value.len();
            let mut pos = 0;
            while pos < max_pos{
                let elm = vec_value[pos];
                if elm.ends_with(r"\"){
                    let elm = elm.replace(r"\", "");
                    if let Some(part) = vec_value.get(pos+1){
                        let merged_arg = format!("{elm}.{part}");
                        args.push(merged_arg);
                        // skip next iteam as it's been merged
                        pos += 1;
                    }
                    
                }else{
                    args.push(elm.to_string());
                }
                pos +=1;
            };
            return Ok(Self{args: args, delimiter: Some(DOT.to_owned())})
        }
        let mut delimiter: Option<String>;
        if let Some(_) = value.find(&DOT){
            delimiter = Some(DOT.to_owned());
        }else if let Some(_) = value.find(&SLASH){
            delimiter = Some(SLASH.to_owned());
        }else{
            delimiter = None
        }
        
        let args: Vec<String> = match delimiter.clone(){
            Some(del)=> value.split(&del).map(|s| s.to_owned()).collect(),
            None => vec![value]
        };
        Ok(Self { args: args, delimiter: delimiter })
        
    }
}

enum ReturnType{
    STRING,
    INT,
    NONE
}

impl From<String> for ReturnType{
    fn from(value: String) -> Self {
        return match value.as_str(){
            "str" => ReturnType::STRING,
            "int" => ReturnType::INT,
            _ => ReturnType::NONE
        }
    }
}
/* 
Args:
data (dict | list): Input dictionary to be searched in.
path (str, optional): Dictionary key search path (pathsep separated).
    Defaults to None.
default (Any, optional): Default value to return if the key is not found.
    Defaults to None.
checknone (bool, optional): If set, an exception is thrown if the value
    is None. Defaults to False.
ignorecase (bool, optional): If set, upper/lower-case keys are treated
    the same. Defaults to False.
pathsep (str, optional): Path separator for path parameter. Defaults to ".".
rtype=None,
*/
#[pyfunction]
fn dictor(_py: Python, 
    data: & PyAny,
    path: Option<String>, 
    default: Option<&str>,
    checknone: Option<bool>,
    ignorecase: Option<bool>,
    pathsep: Option<String>,
    search: Option<String>,
    rtype: Option<String> // TODO: test if movable to Enum Int/String
) -> PyResult<Option<PyObject>> {
    let mut inner_object: &PyAny = pyo3::PyTryInto::try_into(data).unwrap();
    let input: Input;
    let ignorecase = ignorecase.unwrap_or(false);
    let mut found = false;
    let mut return_type: ReturnType;
    let mut int_type: PyInt;
    if rtype.is_some(){
        return_type = rtype.unwrap().into();
    }else{
        return_type = ReturnType::NONE;
    }
    
    if path.is_none() && search.is_none(){
        return Ok(None)
    }

    if path.is_some(){
        let path = path.clone().unwrap();
    
        if let Some(delimiter) = pathsep{
            input = Input::new(path, delimiter);
    
        }else{
            input = match Input::try_from(path) {
                Ok(input) => input,
                Err(e) => Err(PyErr::new::<PyTypeError, _>(e.to_string()))?
            };
        };
        
        let mut inner_item: Result<&PyAny, PyErr>;
        
        if !input.args.is_empty(){
            
            for mut arg in input.args.into_iter(){
                if inner_object.is_instance_of::<PyDict>(){
                    inner_object = inner_object.downcast::<PyDict>().unwrap();
                } else if inner_object.is_instance_of::<PyList>(){
                    inner_object = inner_object.downcast::<PyList>().unwrap();
                } else{
                    if let Some(default_resp) = default{
                        let resp = PyString::new(_py, default_resp);
                        return Ok(Some(resp.to_object(_py)));
                    }else{
                        return Ok(None)
                    }
                }

                if ignorecase {
                    if inner_object.is_instance_of::<PyDict>(){
                        let inner_dict = inner_object.downcast::<PyDict>().unwrap();
                        let cased_key= inner_dict.keys().iter()
                        .filter(|k|{
                            k.to_string().to_lowercase() == arg.to_lowercase()
                        })
                        .map(|k| k.to_string())
                        .collect::<Vec<String>>();
                        if let Some(key) = cased_key.first(){
                            arg = key.to_owned();
                        } else{
                            if let Some(default_resp) = default{
                                let resp = PyString::new(_py, default_resp);
                                return Ok(Some(resp.to_object(_py)));
                            } else{
                                if checknone.is_some_and(|v|v){
                                    return Err(PyErr::new::<PyValueError, _>("value not found for search path"));
                                }
                                return Ok(None);
                            }
                        }
                    }
                };
                // if arg is int (as string) it means we are dealing with a list (or supposing it)
                // due to the int arg
                if let Ok(num_arg) = arg.parse::<i32>(){
                    inner_item  = inner_object.get_item(num_arg);
                    if inner_item.is_err(){
                        inner_item = inner_object.get_item(arg);
                    }
                }
                else{
                    inner_item = inner_object.get_item(arg);
                };
                if let Ok(item) = inner_item{
                    inner_object = item;
                    found = true;
                }else{
                    if let Some(default_resp) = default{
                        let resp = PyString::new(_py, default_resp);
                        return Ok(Some(resp.to_object(_py)));
                    }else if checknone.is_some_and(|v|v){
                        return Err(PyErr::new::<PyValueError, _>("value not found for search path"));
                    }else {
                        return Ok(None);
                    }
                    
                };
            }
        }
    }
    if search.is_some() &&  !inner_object.is_none(){
        let accumulator: Vec<PyAny> = vec![];
        let py_list_accumulator = PyList::new(_py, accumulator);
        find_occurences(_py, search.unwrap().as_str(), inner_object, default, py_list_accumulator);
        if py_list_accumulator.is_empty() && checknone.is_some_and(|v|v){
            return Err(PyErr::new::<PyValueError, _>(format!("value not found for search path: {:?}", path)));
        }else{
            return Ok(Some(py_list_accumulator.to_object(_py)));
        }
        
        }

     
    if !found && default.is_some(){
        let default_resp = PyString::new(_py, default.unwrap());
        Ok(Some(default_resp.into()))
    }else if !found && checknone.is_some_and(|v|v) && inner_object.is_none(){
        return Err(PyValueError::new_err(format!("value not found for search path: {:?}", path)));

    }else{
        // cast to return type if no errors, keep original type otherwise
        match return_type{
            ReturnType::STRING =>{
                // ignore if cannot cast and keep original format
                let casted_matching_item = inner_object.str();
                if casted_matching_item.is_ok(){
                    inner_object = casted_matching_item.unwrap();
                }
               
            },
            ReturnType::INT =>{
                let content_str: Result<String, PyErr> = inner_object.extract();
                // All this nasty hack is to overcome the issue:
                // https://github.com/PyO3/pyo3/issues/2221
                if let Ok(content) = content_str{
                    let numeric_content = content.parse::<usize>();
                    if let Ok(num_inner_object) = numeric_content{
                        let asd = PyFloat::new(_py, num_inner_object as f64);
                        inner_object = asd.into();
                    }
                }
            },
            ReturnType::NONE=>{} // keep original value
        };
        Ok(Some(inner_object.into()))
    }
    
}


fn find_occurences(py: Python, target: &str, searchable: &PyAny, default: Option<&str>, accumulator: &PyList){
    if searchable.is_instance_of::<PyList>(){
        let iter = searchable.iter().unwrap();
        for maybe_element in iter {
            if let Ok(element) = maybe_element{
                find_occurences(py, &target, element, default, accumulator);

            }
        }
    }else if searchable.is_instance_of::<PyDict>(){
        
        let inner_dict = searchable.downcast::<PyDict>().unwrap();
        for key in inner_dict.keys(){
           
            if let Some(matching_item) = inner_dict.get_item(key){
                if key.to_string() == target{
                    let obj_type = matching_item.get_type();
                    let bool_type = py.get_type::<PyBool>();
                    let str_type = py.get_type::<PyString>();
                    if obj_type.is(bool_type){
                        accumulator.append(matching_item).unwrap();
                    }else if obj_type.is(str_type) {
                        accumulator.append(matching_item).unwrap();
                    }else if default.is_some(){
                        accumulator.append(default).unwrap();
                    }else{
                        accumulator.append(matching_item).unwrap();
                    }
                }else if matching_item.is_instance_of::<PyDict>() ||
                matching_item.is_instance_of::<PyList>(){
                    find_occurences(py, target, matching_item, default, accumulator)
                }

            }
        }
    }

}

#[pymodule]
pub fn dicto_r(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(dictor, _py)?)?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use pyo3::types::{PyDict, PyList};
    use pyo3::Python;

    use super::*;

    #[test]
    fn test_missing_key_return_default(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let list_dict = py.eval("[ \
            {'name': name, 'genre': genre, 'status': status} \
            for name, genre,status in [ \
                ('spaceballs', 'comedy', False), \
                ('gone with the wind', 'tragedy', ''), \
                ('titanic', 'comedy', True), \
                (None, 'comedy', None), \
            ]]", None, None).unwrap();
            let res = dictor(py, list_dict, None, Some("pepe"),None, 
                None, None, Some("name".to_string()), None);
            let expected = PyList::new(py,vec!["spaceballs", "gone with the wind", "titanic", "pepe"]);
            let content = res.unwrap().unwrap();
            let content: &PyList = content.downcast(py).unwrap();
            assert!(expected.compare(content).is_ok());
            
           
                
        });
    }
    #[test]
    fn find_null_key_ignore_default(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py|{
             // find null key
             let dict = py.eval("{ \
                'terminator': [ \
                    { \
                        'terminator 1': { \
                            'year': 1987, \
                            'status': 'Nested Status', \
                            'genre': [ \
                                'futureshock', \
                                'scifi' \
                            ] \
                        } \
                    }, \
                    { \
                        'terminator 2': { \
                            'year': 1992, \
                            'genre': [ \
                                'nuclear war', \
                                'scifi' \
                            ] \
                        } \
                    }, \
                    { \
                        'terminator 3': { \
                            'year': 0, \
                            'available': 'false', \
                            'stars': '', \
                            'preview': None \
                        }
                    }
                ]
            }", None, None).unwrap();
            let res = dictor(py, dict, 
                Some("terminator.2.terminator 3.preview".to_owned()),
                Some("pepe"),None, 
                None, None, None, None);
            let content = res.unwrap().unwrap();
            assert!(content.is_none(py))

        });
    }

    #[test]
    fn replace_int_to_str(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py|{
             // find null key
            let dict = py.eval("\
                { \
                    'year': '1983', \
                    'status': 'Nested Status' \
                    }", None, None).unwrap(); 

            let res = dictor(py, dict, 
                Some("year".to_owned()),
                None,None, 
                None, None, None, Some("str".into())).unwrap();
            let content = res.unwrap();
            let content = content.downcast::<PyString>(py).unwrap();
            let expected_content = PyString::new(py, "1983");
            assert!(content.eq(expected_content).unwrap())
            });
    }

    #[test]
    fn replace_int_to_int(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py|{
            let dict = py.eval("\
                { \
                    'year': 1987, \
                    'status': 'Nested Status' \
                    }", None, None).unwrap(); 

            let res = dictor(py, dict, 
                Some("year".to_owned()),
                None,None, 
                None, None, None, Some("int".into())).unwrap();
            let content = res.unwrap();
            let content: usize = content.extract(py).unwrap();
            assert!(content == 1987)
            });
    }
    
    #[test]
    fn replace_str_to_int(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py|{
            let dict = py.eval("\
                { \
                    'year': '1987', \
                    'status': 'Nested Status' \
                    }", None, None).unwrap(); 

            let res = dictor(py, dict, 
                Some("year".to_owned()),
                None,None, 
                None, None, None, Some("int".into())).unwrap();
            let content = res.unwrap();
            let content: f32 = content.extract(py).unwrap();
            assert!(content == 1987.0)
            });
    }
 

    #[test]
    fn find_occurences_test(){
        // ejemplo facil:
        // lista de jsons, busco un valor:
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let mut elements: Vec<&PyDict> = vec![];
            for elm in ["pepe", "pipo", "popo"].into_iter(){
                let elm1 = PyDict::new(py);
                elm1.set_item("name", elm);
                elm1.set_item("last_name", format!("{elm}_last_name"));
                elements.push(elm1);
            }

            // add a None valued item to be replaced by default arg
            let elm1 = PyDict::new(py);
            let val: Option<String>;
            val = None;
            elm1.set_item("name", val);
            elm1.set_item("last_name", format!("no_last_name"));
            elements.push(elm1);

            let elm1 = PyDict::new(py);
            elm1.set_item("name", "papa");
            let elements2 : Vec<&PyDict> = vec![elm1];
            let elm2 = PyDict::new(py);
            elm2.set_item("pepe", elements2);
            elements.push(elm2);
            let vec_accumulator : Vec<PyString>= vec![];
            let base_list = PyList::new(py, elements);
            let accumulator = PyList::new(py, vec_accumulator);
            find_occurences(py, "name", &base_list, Some("default"), accumulator);
            let expected = PyList::new(py,vec!["pepe", "pipo", "popo", "papa", "default"]);
            assert!(accumulator.compare(expected).is_ok());
        });
        
    }
       
    #[test]
    fn test_searching_list_JSON(){

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let list_dict = py.eval("[ \
                {'name': name, 'genre': genre, 'status': status} \
                for name, genre,status in [ \
                    ('spaceballs', 'comedy', False), \
                    ('gone with the wind', 'tragedy', ''), \
                    ('titanic', 'comedy', True), \
                    ('titanic', 'comedy', None), \
                ]]", None, None).unwrap();
            let res = dictor(py, list_dict, None, None,None, 
                None, None, Some("name".to_string()), None);
            let expected = PyList::new(py,vec!["spaceballs", "gone with the wind", "titanic", "titanic"]);
            let content = res.unwrap().unwrap();
            assert!(expected.compare(content).is_ok());
        });
       
    }


    #[test]
    fn test_int_as_string(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("item", vec![1,2,3]).unwrap();
            // test index out of range should be silenced
            let res = dictor(
                py, dict, Some("item.4".into()), 
                None, None, 
                None, None, None, None);
            assert!(res.unwrap().is_none());

            let dict2 = PyDict::new(py);
            dict2.set_item("4", "found");
            dict.set_item("other_item", dict2);
            let res = dictor(
                py, dict, Some("other_item.4".into()), 
                None, None, 
                None, None, None, None).unwrap();
            assert_eq!(res.unwrap().to_string() , "found".to_string());

         
        })
    }
        

    #[test]
    fn tupl_value(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py|{
            let dict = py.eval("\
            { \
                'name': 'joe', \
                'age': 32, \
                'hobbies': ('skiing', 'archery', 'chess'), \
                'foods': ['spam', 'celery', ('milk', 'cheese', 'yogurt'), 'cake'], \
                'key1': { \
                    'foods': ['carrot', 'potato'], \
                    'subkey1': { \
                        'foods': ('pear', 'cherry') \
                    } \
                } \
            }", None, None).unwrap(); 

            let res = dictor(py, dict, 
                None,
                None,None, 
                None, None, Some("foods".into()), None).unwrap();
            let content = res.unwrap();
            // I have no idea how to convert this object but from python's side
            // it runs Ok
            });
        
    }
    
    #[test]
    fn test_escape_pathsep(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py|{
            let dict = py.eval("{\
                'dirty.harry': { \
                    'year': 1977, \
                    'genre': 'romance', \
                    'status': '' \
                }\
            }", None, None).unwrap();
            let res = dictor(py, dict, 
                Some(r"dirty\.harry.genre".into()),
                None,None, 
                None, None, None, None).unwrap();
            let content = res.unwrap();
            assert_eq!(content.to_string(), "romance");

        });
    }

    #[test]
    fn test_cased_target(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            let dict2 = PyDict::new(py);
            dict2.set_item("aLGO", "found");
            dict.set_item("oTRo", dict2);
            
            let res = dictor(py, dict, Some("otro.algo".to_string()),
             None, None,Some(true), None, None, None);
            assert_eq!(res.unwrap().to_object(py).to_string(), "found".to_string());
        });
    }

    #[test]
    fn test_default_ignorecase(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            let dict2 = PyDict::new(py);
            dict2.set_item("aLGO", "found");
            dict.set_item("oTRo", dict2);
            
            let res = dictor(py, dict, Some("otro.nonexistent".to_string()),
             Some("replaced"), None,Some(true), None, None, None);
            assert_eq!(res.unwrap().to_object(py).to_string(), "replaced".to_string());
        });
    }
    #[test]
    fn test_default(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            let dict2 = PyDict::new(py);
            dict2.set_item("algo", "found");
            dict.set_item("otro", dict2);
            
            let res = dictor(py, dict, Some("otro.nonexistent".to_string()),
             Some("replaced"), None,Some(true), None, None, None);
            assert_eq!(res.unwrap().to_object(py).to_string(), "replaced".to_string());
        });
    }

    #[test]
    fn test_search(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let dict1 = PyDict::new(py);
            let dict2 = PyDict::new(py);
            let dict3 = PyDict::new(py);
            dict1.set_item("some_key", "value1").unwrap();
            dict2.set_item("some_key", "value_2").unwrap();
            dict3.set_item("some_key", "value_3").unwrap();
            let list: &PyList = PyList::new(py, vec![dict1, dict2, dict3]);
        
            let res = dictor(py, list, Some("otro.algo".to_string()),
             None, None,Some(true), None, Some("some_key".to_string()), None);
            let content = res.unwrap();
            assert!(content.is_none());
        });
    }


    #[test]
    fn test_raise_exception(){

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let list_dict = py.eval("[ \
                {'name': name, 'genre': genre, 'status': status} \
                for name, genre,status in [ \
                    ('spaceballs', 'comedy', False), \
                    ('gone with the wind', 'tragedy', ''), \
                    ('titanic', 'comedy', True), \
                    ('titanic', 'comedy', None), \
                ]]", None, None).unwrap();
            let res = dictor(py, list_dict, 
                Some("8.sarasa".to_owned()), None, Some(true), 
                None, None, None, None);
            
        });
    }
}