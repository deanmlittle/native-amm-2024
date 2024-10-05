use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TryFromBytes)]
pub fn derive_try_from_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl TryFrom<&[u8]> for #name {
            type Error = ProgramError;
        
            fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
                bytemuck::try_pod_read_unaligned::<Self>(data)
                    .map_err(|_| ProgramError::InvalidInstructionData)
            }
        }
    };
    
    TokenStream::from(expanded)
}