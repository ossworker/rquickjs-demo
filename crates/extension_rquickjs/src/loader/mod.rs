use std::collections::HashMap;

use rquickjs::{Ctx, Error, Module, Object, Result, loader::Loader, module::Exports};

type ModuleLoadFn = for<'js> fn(ctx: Ctx<'js>, Vec<u8>) -> Result<Module<'js>>;

pub struct ModuleLoader {
    modules: HashMap<&'static str, ModuleLoadFn>,
}

impl ModuleLoader {
    pub fn new(modules: HashMap<&'static str, ModuleLoadFn>) -> Self {
        Self { modules }
    }
}

impl Loader for ModuleLoader {
    fn load<'js>(
        &mut self,
        ctx: &Ctx<'js>,
        path: &str,
    ) -> Result<Module<'js, rquickjs::module::Declared>> {
        let load = self
            .modules
            .remove(path)
            .ok_or_else(|| Error::new_loading(path))?;

        (load)(ctx.clone(), Vec::from(path))
    }
}

pub(crate) fn export_default<'js, F>(
    ctx: &Ctx<'js>,
    exports: &Exports<'js>,
    f: F,
) -> rquickjs::Result<()>
where
    F: FnOnce(&Object<'js>) -> rquickjs::Result<()>,
{
    let default = Object::new(ctx.clone())?;
    f(&default)?;

    for object_key in default.keys::<String>() {
        let name = object_key?;
        let value: rquickjs::Value = default.get(&name)?;
        exports.export(name, value)?;
    }

    exports.export("default", default)?;

    Ok(())
}
