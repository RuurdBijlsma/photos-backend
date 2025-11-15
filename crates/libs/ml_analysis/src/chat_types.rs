use pyo3::{Bound, IntoPyObject, IntoPyObjectExt, PyAny, Python};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    Assistant,
    User,
}

impl<'py> IntoPyObject<'py> for ChatRole {
    type Target = PyAny; // the Python type
    type Output = Bound<'py, Self::Target>; // in most cases this will be `Bound`
    type Error = std::convert::Infallible; // the conversion error type, has to be convertible to `PyErr`

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Self::User => {
                let result = "user"
                    .into_bound_py_any(py)
                    .expect("Can't bind string to Python.");
                Ok(result)
            }
            Self::Assistant => {
                let result = "assistant"
                    .into_bound_py_any(py)
                    .expect("Can't bind string to Python.");
                Ok(result)
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, IntoPyObject)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}
