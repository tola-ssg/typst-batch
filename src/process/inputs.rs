//! Shared `with_inputs` methods via trait.

use typst::foundations::{Dict, IntoValue, Str};

use crate::codegen::Inputs;

/// Trait for types that accept `sys.inputs` configuration.
///
/// Provides `with_inputs`, `with_inputs_dict`, and `with_inputs_obj` methods.
pub trait WithInputs: Sized {
    /// Get mutable reference to the inputs field.
    fn inputs_mut(&mut self) -> &mut Option<Dict>;

    /// Set `sys.inputs` from key-value pairs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let compiler = Compiler::new(root)
    ///     .with_inputs([("key", "value"), ("draft", true)]);
    /// ```
    fn with_inputs<I, K, V>(mut self, inputs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Str>,
        V: IntoValue,
    {
        let dict: Dict = inputs
            .into_iter()
            .map(|(k, v)| (k.into(), v.into_value()))
            .collect();
        *self.inputs_mut() = Some(dict);
        self
    }

    /// Set `sys.inputs` from a pre-built Dict.
    fn with_inputs_dict(mut self, inputs: Dict) -> Self {
        *self.inputs_mut() = Some(inputs);
        self
    }

    /// Set `sys.inputs` from an [`Inputs`] object.
    ///
    /// Use [`Inputs::from_json()`] or [`Inputs::from_json_with_content()`]
    /// to create the inputs.
    fn with_inputs_obj(mut self, inputs: Inputs) -> Self {
        *self.inputs_mut() = Some(inputs.into_dict());
        self
    }
}
