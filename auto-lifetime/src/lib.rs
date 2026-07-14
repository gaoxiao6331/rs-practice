use proc_macro::TokenStream;

use quote::quote;

use proc_macro2::Span;

use std::collections::{HashMap, HashSet};


use syn::{
    parse_macro_input,

    ItemMod,
    Item,

    GenericParam,
    Lifetime,
    LifetimeParam,

    Type,
    TypePath,
    TypeReference,

    PathArguments,
    GenericArgument,

    visit::Visit,
    visit_mut::VisitMut,
};



#[proc_macro_attribute]
pub fn auto_lifetime(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {


    let mut module =
        parse_macro_input!(
            item as ItemMod
        );


    expand(
        &mut module
    );


    TokenStream::from(
        quote!(#module)
    )
}



fn expand(
    module:&mut ItemMod
){

    let items =
        match &mut module.content {

            Some((_,items)) => items,

            None => return,
        };



    //
    // 收集类型
    //
    let mut names =
        HashSet::new();



    for item in items.iter(){

        match item {

            Item::Struct(s)=>{
                names.insert(
                    s.ident.to_string()
                );
            }

            Item::Enum(e)=>{
                names.insert(
                    e.ident.to_string()
                );
            }

            _=>{}
        }
    }





    //
    // 类型依赖图
    //
    let mut deps:
        HashMap<String,HashSet<String>>
        = HashMap::new();



    for item in items.iter(){


        if let Some(name)=item_name(item){


            let mut finder =
                DependencyFinder{
                    result:HashSet::new()
                };


            finder.visit_item(item);


            deps.insert(
                name,
                finder.result
            );
        }

    }





    //
    // 找出直接有引用的类型
    //
    let mut need =
        HashSet::new();



    for item in items.iter(){

        if has_reference(item){

            if let Some(name)=item_name(item){

                need.insert(name);

            }
        }
    }





    //
    // DFS传播
    //
    loop {

        let mut changed=false;


        for (name,children)
        in deps.iter()
        {

            if need.contains(name){
                continue;
            }


            if children.iter()
                .any(|x|need.contains(x))
            {

                need.insert(
                    name.clone()
                );

                changed=true;
            }
        }


        if !changed{
            break;
        }
    }





    //
    // 修改
    //
    for item in items.iter_mut(){


        let name =
            match item_name(item){

                Some(x)=>x,

                None=>continue,
            };



        if !need.contains(&name){
            continue;
        }



        add_lifetime(item);


        let mut rw =
            LifetimeRewriter{
                targets:&need,
                lifetime:
                Lifetime::new(
                    "'a",
                    Span::call_site()
                )
            };


        rw.visit_item_mut(item);

    }

}





fn item_name(
    item:&Item
)->Option<String>{

    match item {

        Item::Struct(s)=>
            Some(
                s.ident.to_string()
            ),


        Item::Enum(e)=>
            Some(
                e.ident.to_string()
            ),


        _=>None
    }
}





fn add_lifetime(
    item:&mut Item
){

    let generics =
        match item {

            Item::Struct(s)=>
                &mut s.generics,


            Item::Enum(e)=>
                &mut e.generics,


            _=>return,
        };



    if generics.lifetimes()
        .next()
        .is_some()
    {
        return;
    }



    generics.params.push(
        GenericParam::Lifetime(
            LifetimeParam::new(
                Lifetime::new(
                    "'a",
                    Span::call_site()
                )
            )
        )
    );
}







fn has_reference(
    item:&Item
)->bool{


    struct Finder{
        found:bool
    }



    impl<'ast> Visit<'ast>
    for Finder{


        fn visit_type_reference(
            &mut self,
            node:&'ast TypeReference
        ){

            self.found=true;


            syn::visit::visit_type_reference(
                self,
                node
            );
        }

    }



    let mut f =
        Finder{
            found:false
        };


    f.visit_item(item);


    f.found
}







struct DependencyFinder {

    result:HashSet<String>,
}



impl<'ast> Visit<'ast>
for DependencyFinder{


    fn visit_type_path(
        &mut self,
        node:&'ast TypePath
    ){


        if let Some(seg)=
            node.path.segments.last()
        {

            let name =
                seg.ident.to_string();


            self.result.insert(name);

        }


        syn::visit::visit_type_path(
            self,
            node
        );
    }
}







struct LifetimeRewriter<'a>{

    targets:&'a HashSet<String>,

    lifetime:Lifetime,
}



impl VisitMut
for LifetimeRewriter<'_>{



    fn visit_type_reference_mut(
        &mut self,
        node:&mut TypeReference
    ){

        if node.lifetime.is_none(){

            node.lifetime =
                Some(
                    self.lifetime.clone()
                );
        }


        syn::visit_mut::
        visit_type_reference_mut(
            self,
            node
        );
    }






    fn visit_type_path_mut(
        &mut self,
        node:&mut TypePath
    ){


        if let Some(seg)=
            node.path.segments.last_mut()
        {


            let name =
                seg.ident.to_string();



            if self.targets.contains(&name)
                &&
                matches!(
                    seg.arguments,
                    PathArguments::None
                )
            {


                let mut args =
                    syn::punctuated::Punctuated::new();


                args.push(
                    GenericArgument::Lifetime(
                        self.lifetime.clone()
                    )
                );


                seg.arguments =
                    PathArguments::AngleBracketed(
                        syn::AngleBracketedGenericArguments{

                            colon2_token:None,

                            lt_token:
                            syn::token::Lt(
                                Span::call_site()
                            ),

                            args,

                            gt_token:
                            syn::token::Gt(
                                Span::call_site()
                            ),
                        }
                    );

            }

        }



        syn::visit_mut::
        visit_type_path_mut(
            self,
            node
        );
    }

}