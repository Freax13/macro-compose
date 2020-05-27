use proc_macro2::TokenStream;
use quote::ToTokens;
use std::ops::Deref;
use syn::{parse, parse::Parse, parse2, Error};

use crate::{Expand, Lint};

/// Collector collects the results and errors of a macro expansion
pub struct Collector {
    err_count: usize,
    output: TokenStream,
}

impl Collector {
    /// create a new collector
    pub fn new() -> Self {
        Collector {
            err_count: 0,
            output: TokenStream::new(),
        }
    }

    /// report an error
    ///
    /// once an error has been reported to an collector, `Expand`s will no longer be run
    pub fn error(&mut self, e: Error) {
        let error: TokenStream = e.to_compile_error();
        self.output.extend(error);
        self.err_count += 1;
    }

    /// checks if any errors have been reported yet
    pub fn has_errors(&self) -> bool {
        self.err_count != 0
    }

    /// finish the expansion and return the result
    pub fn finish(self) -> TokenStream {
        self.output
    }
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

enum Data<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}

impl<T> Deref for Data<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Data::Owned(data) => &data,
            Data::Borrowed(data) => data,
        }
    }
}

/// used to lint, expand and capture
///
/// a context might not have data in it (eg. if parsing the data failed), so calling the lint and expand functions does not guarantee the `Lint`s and `Expand`s actually run
pub struct Context<'a, T> {
    collector: &'a mut Collector,
    data: Option<Data<'a, T>>,
}

impl<'a, T> Context<'a, T> {
    /// create a new context with the data
    pub fn new(collector: &'a mut Collector, data: T) -> Self {
        Context {
            collector,
            data: Some(Data::Owned(data)),
        }
    }

    /// create a new context with the data passed by reference
    pub fn new_by_ref(collector: &'a mut Collector, data: &'a T) -> Self {
        Context {
            collector,
            data: Some(Data::Borrowed(data)),
        }
    }

    /// create a new context without any data in it
    pub fn new_empty(collector: &'a mut Collector) -> Self {
        Context {
            collector,
            data: None,
        }
    }

    /// try to parse the data from a [`proc_macro::TokenStream`]
    ///
    /// if parsing the data fails the error is reported to the collector
    pub fn new_parse(collector: &'a mut Collector, data: proc_macro::TokenStream) -> Self
    where
        T: Parse,
    {
        match parse::<T>(data) {
            Ok(data) => Self::new(collector, data),
            Err(e) => {
                collector.error(e);
                Self {
                    collector,
                    data: None,
                }
            }
        }
    }

    /// try to parse the data from a [`proc_macro2::TokenStream`]
    ///
    /// if parsing the data fails the error is reported to the collector
    pub fn new_parse2(collector: &'a mut Collector, data: TokenStream) -> Self
    where
        T: Parse,
    {
        match parse2::<T>(data) {
            Ok(data) => Self::new(collector, data),
            Err(e) => {
                collector.error(e);
                Self {
                    collector,
                    data: None,
                }
            }
        }
    }

    /// lint the macro input
    ///
    /// returns true if the lint ran without reporting an error
    pub fn lint<L: Lint<T>>(&mut self, lint: &L) -> bool {
        if let Some(data) = self.data.take() {
            let start = self.collector.err_count;
            lint.lint(&*data, &mut self.collector);
            self.data = Some(data);
            start == self.collector.err_count
        } else {
            false
        }
    }

    /// expand the macro and add the result to the collector
    pub fn expand(&mut self, expand: &impl Expand<T>) {
        if let Some(res) = self.capture(expand) {
            let tts: TokenStream = res.to_token_stream();
            self.collector.output.extend(tts);
        }
    }

    /// expand the macro and return the output
    pub fn capture<E: Expand<T>>(&mut self, expand: &E) -> Option<E::Output> {
        if self.collector.has_errors() {
            return None;
        }
        if let Some(data) = self.data.as_ref() {
            expand.expand(&*data, &mut self.collector)
        } else {
            None
        }
    }
}
