use crate::{
    codegen::{
        get_destructured_imports, get_endian,
        sanitization::{ARGS, OPT, WRITER, WRITE_METHOD},
    },
    parser::{CondEndian, Input, Magic},
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub(crate) struct PreludeGenerator<'a> {
    out: TokenStream,
    input: Option<&'a Input>,
    name: Option<&'a Ident>,
}

impl<'a> PreludeGenerator<'a> {
    pub(crate) fn new(out: TokenStream, input: Option<&'a Input>, name: Option<&'a Ident>) -> Self {
        Self { out, input, name }
    }

    pub(crate) fn prefix_imports(mut self) -> Self {
        if let Some(imports) = self
            .input
            .and_then(|input| get_destructured_imports(input.imports(), self.name, true))
        {
            let out = self.out;
            self.out = quote! {
                let #imports = #ARGS;
                #out
            };
        }

        self
    }

    pub(crate) fn prefix_magic(mut self, magic: &Magic) -> Self {
        if let Some(magic) = magic {
            let magic = magic.match_value();
            let out = self.out;
            self.out = quote! {
                #WRITE_METHOD (
                    &#magic,
                    #WRITER,
                    #OPT,
                    ()
                )?;

                #out
            };
        }

        self
    }

    pub(crate) fn prefix_endian(mut self, endian: &CondEndian) -> Self {
        let endian = get_endian(endian);
        let out = self.out;
        self.out = quote! {
            let #OPT = #endian;
            #out
        };

        self
    }

    pub(crate) fn finish(self) -> TokenStream {
        self.out
    }
}
