//! An elegant dictionary and JSON handler (Rust implementation of python's lib dictor)
//! 
//! Dictor is a Python JSON and Dictionary (Hash, Map) handler.
//! Dictor takes a dictionary or JSON data and returns value for a specific key.
//! 
//! If Dictor doesnt find a value for a key, or if JSON or Dictionary data is missing the key, the return value is either None or whatever fallback value you provide.
//! 
//!Dictor is polite with Exception errors commonly encountered when parsing large Dictionaries/JSONs.
//!Using Dictor eliminates the repeated use of try/except blocks in your code when dealing with lookups of large JSON structures, as well as providing flexibility for inserting fallback values on missing keys/values.


use std::any::Any;
use std::default;
use std::fmt::Display;
use std::time::Instant;

use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyString, PyList};
use pyo3::{ToPyObject, PyAny, PyErr};
use pyo3::{types::{PyDict, PyModule}, PyResult, pymodule, Python, PyObject, exceptions::PyValueError,
wrap_pyfunction, pyfunction};


const DOT: &str = ".";
const SLASH: &str = "/";


#[derive(Debug)]
pub struct Input{
    args: Vec<String>,
    delimiter: String
}
impl Input {
    fn new(raw_input: String, delimiter: String) -> Self {
        let args: Vec<String> = raw_input.split(&delimiter).map(|s| s.to_owned()).collect();
        Self { args: args, delimiter: delimiter }
    }
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
        let mut delimiter: String = "".into();
        if let Some(_) = value.find(&DOT){
            delimiter = DOT.to_owned();
        }else if let Some(_) = value.find(&SLASH){
            delimiter = SLASH.to_owned();
        }else{
            return Err(ParseError::InvalidDelimiter(delimiter))
        }
        let args: Vec<String> = value.split(&delimiter).map(|s| s.to_owned()).collect();
        Ok(Self { args: args, delimiter: delimiter })
        
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
*/
#[pyfunction]
fn dictor(_py: Python, 
    data: & PyAny,
    path: String, 
    default: Option<&str>,
    checknone: Option<bool>,
    ignorecase: Option<bool>,
    pathsep: Option<String>,
    search: Option<String>
) -> PyResult<Option<PyObject>> {
    let mut inner_object: &PyAny = data.try_into().unwrap();
    let input: Input;
    let ignorecase = ignorecase.unwrap_or(false);

    if let Some(delimiter) = pathsep{
        input = Input::new(path, delimiter);
   
    }else{
        input = match Input::try_from(path) {
            Ok(input) => input,
            Err(e) => Err(PyErr::new::<PyTypeError, _>(e.to_string()))?
        };
    };
    let accumulator: Vec<&PyAny> = vec![];
    let mut inner_item: Result<&PyAny, PyErr>;
    
    if !input.args.is_empty(){
        for mut arg in input.args.into_iter(){
            if inner_object.is_instance_of::<PyDict>(){
                inner_object = inner_object.downcast::<PyDict>().unwrap();
            } else if inner_object.is_instance_of::<PyList>(){
                inner_object = inner_object.downcast::<PyList>().unwrap();
            } else{
                // if can't keep iterating due to not iterator, then we couldn't find the value
                // so either default or None value is returned
                if let Some(default_resp) = default{
                    let resp = PyString::new(_py, default_resp);
                    return Ok(Some(resp.to_object(_py)));
                }else{
                    return Ok(None)
                }

            if ignorecase {
                // filter by keys
                // if found in key, override key with original value and continue
                // if not dict, do not do anythin
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
                            return Ok(None);
                        }
                    }
                }
            };
            if let Ok(num_arg) = arg.parse::<i32>(){
                inner_item  = inner_object.get_item(num_arg);
                if inner_item.is_err(){
                    inner_item = inner_object.get_item(arg);
                    // if item matches search, attach to accumulator prior overriding
                }
            }
            else{
                inner_item = inner_object.get_item(arg);
            };
            
            if let Ok(item) = inner_item{

                inner_object = item;
                
            }else{
                if let Some(default_resp) = default{
                    let resp = PyString::new(_py, default_resp);
                    return Ok(Some(resp.to_object(_py)));
                }else{
                    return Ok(None);
                }
                
            };
        }
    }
    if search.is_some() &&  !inner_object.is_none(){
        todo!("call find_occurrences")
        }

     
    }
    //TODO checknone: if None is found and `checknone`` set a PyErr must be raised
    if inner_object.is_none() && default.is_some(){
        let default_resp = PyString::new(_py, default.unwrap());
        Ok(Some(default_resp.to_object(_py)))
    }else {
        Ok(Some(inner_object.to_object(_py)))
    }
    
}


fn find_occurences(py: Python, target: &str, searchable: &PyAny, accumulator: &PyList){
    dbg!(searchable);
    if searchable.is_instance_of::<PyList>(){
        let iter = searchable.iter().unwrap();
        for maybe_element in iter {
            if let Ok(element) = maybe_element{
                find_occurences(py, &target, element, accumulator);
               
            }
        }
    }else if searchable.is_instance_of::<PyDict>(){
        
        let inner_dict = searchable.downcast::<PyDict>().unwrap();
        for key in inner_dict.keys(){
            if let Some(matching_item) = inner_dict.get_item(key){
                if matching_item.is_instance_of::<PyDict>() ||
                matching_item.is_instance_of::<PyList>(){
                    find_occurences(py, target, matching_item, accumulator)
                }else{
                    if matching_item.is_instance_of::<PyString>() && key.to_string() == target{
                        let string = matching_item.downcast::<PyString>().unwrap();
                        accumulator.append(string);
                    } 
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
    fn find_occurences_test(){
        // ejemplo facil:
        // lista de jsons, busco un valor:
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let my_string = PyString::new(py, "pepe");
            let content = my_string.to_str();
            let mut elements: Vec<&PyDict> = vec![];
            
            for elm in ["pepe", "pipo", "popo"].into_iter(){
                let elm1 = PyDict::new(py);
                elm1.set_item("name", elm);
                elm1.set_item("last_name", format!("{elm}_last_name"));
                elements.push(elm1);
            }
            let elm1 = PyDict::new(py);
            elm1.set_item("name", "papa");
            let elements2 : Vec<&PyDict> = vec![elm1];
            let elm2 = PyDict::new(py);
            elm2.set_item("pepe", elements2);
            elements.push(elm2);
            let vec_accumulator : Vec<PyString>= vec![];
            let base_list = PyList::new(py, elements);
            let accumulator = PyList::new(py, vec_accumulator);
            find_occurences(py, "name", &base_list, accumulator);
            dbg!(accumulator);
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
                py, dict, "item.4".into(), 
                None, None, 
                None, None, None);
            assert!(res.unwrap().is_none());

            let dict2 = PyDict::new(py);
            dict2.set_item("4", "found");
            dict.set_item("other_item", dict2);
            let res = dictor(
                py, dict, "other_item.4".into(), 
                None, None, 
                None, None, None).unwrap();
            assert_eq!(res.unwrap().to_string() , "found".to_string());

         
        })
    }
        

    #[test]
    fn test_cased_target(){
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            let dict2 = PyDict::new(py);
            dict2.set_item("aLGO", "found");
            dict.set_item("oTRo", dict2);
            
            let res = dictor(py, dict, "otro.algo".to_string(),
             None, None,Some(true), None, None);
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
            
            let res = dictor(py, dict, "otro.nonexistent".to_string(),
             Some("replaced"), None,Some(true), None, None);
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
            
            let res = dictor(py, dict, "otro.nonexistent".to_string(),
             Some("replaced"), None,Some(true), None, None);
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
        
            let res = dictor(py, list, "otro.algo".to_string(),
             None, None,Some(true), None, Some("some_key".to_string()));
            dbg!(res);
            // assert_eq!(res.unwrap().to_object(py).to_string(), "found".to_string());
        });
    }
}