use core::str;

use llrt_modules::{
    os::OsModule,
    path::PathModule,
    url::{self, UrlModule},
};
use rquickjs::{
    async_with, convert, loader::{
        BuiltinLoader, BuiltinResolver, FileResolver, ModuleLoader, ScriptLoader,
    }, module::{Evaluated, ModuleDef}, prelude::Func, promise::{MaybePromise, Promised}, qjs, AsyncContext, AsyncRuntime, CatchResultExt, Context, Ctx, Exception, FromJs as _, Module, Runtime, Value
};

fn print(msg: String) {
    println!("{msg}");
}

async fn delay(millsecond: u32) {
    let duration = std::time::Duration::from_millis(millsecond.into());
    tokio::time::sleep(duration).await;
}

#[cfg(feature = "tokio-sync")]
fn main()->anyhow::Result<()>{
    // let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    // rt.block_on(async_run_js()).unwrap();
    sync_run_js().unwrap();
    Ok(())
}


#[cfg(feature = "tokio-async")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    async_run_js().await
}

 fn sync_run_js()->anyhow::Result<()>{
    let runtime = Runtime::new()?;
    let context = Context::full(&runtime)?;
    let resolver = (
        BuiltinResolver::default()
            .with_module("path")
            .with_module("url")
            .with_module("os")
            .with_module("main")
            .with_module("dayjs"),
        FileResolver::default()
            .with_path("./")
            .with_path("../../target/debug")
            .with_native(),
    );
    

    let loader = (
        BuiltinLoader::default().with_module("dayjs", include_str!("../js/dayjs.mjs")),
        ModuleLoader::default()
        .with_module("path", PathModule)
        .with_module("url", UrlModule)
        .with_module("os", OsModule),
        ScriptLoader::default(),
        // dyn-load
        // NativeLoader::default(), 
    );

    runtime.set_loader(resolver, loader);

    let code = r#"
    import dayjs from 'dayjs'
    import {version,arch,platform} from 'os'
    import url from 'url'
    import {dirname,basename} from 'path'    
    export const handler =  async () => {
        //out> Darwin Kernel Version 24.1.0: Thu Oct 10 21:02:27 PDT 2024; root:xnu-11215.41.3~2/RELEASE_X86_64
        print(version())
        //out> x64
        print(arch())
            
        //out> darwin
        print(platform())

        print(dayjs().format())

        print(typeof url)
        // await delay(1000)
        //out>  workoss
        print(basename('/home/workoss/'))
        return dayjs().format();
    };
    "#;
    let _result = context.with(|ctx| {

        let global = ctx.globals();

        url::init(&ctx).unwrap();

        global.set("print",Func::from(|s|print(s))).unwrap();
        global.set("delay", Func::from(|millsecond|Promised(delay(millsecond)))).unwrap();

        if let Err(e) = Module::declare(ctx.clone(), "main", code).catch(&ctx){
            eprintln!("module declare error:{e:#?}");
        }
       
        // Module::declare(ctx.clone(), "main", code)
        // .catch(&ctx)
        // .map_err(|e|anyhow::anyhow!("declare catch1:{:#?}",e))
        // .unwrap();
        //获取 handler function
        let m: rquickjs::Object = Module::import(&ctx, "main")
        .catch(&ctx)
        .map_err(|e|anyhow::anyhow!("import catch1:{:#?}",e))?
        .finish().unwrap();
        // .into_future()
        // .map_err(|e|anyhow::anyhow!("import catch2:{:#?}",e))?
        ;
        println!("{m:#?}");

        let handler: rquickjs::Function = m.get("handler")
        // .catch(&ctx)
        .map_err(|e|anyhow::anyhow!("get handler:{:#?}",e))?;
        // 执行 handler
        let handler_promise: MaybePromise = handler.call(()).catch(&ctx).map_err(|e|anyhow::anyhow!("call handler:{:#?}",e))?;
        let handler_result = handler_promise.finish::<Value>().catch(&ctx).map_err(|e|anyhow::anyhow!("call result:{:#?}",e))?;
        println!("handler_result:{:#?}",val_to_string(&ctx, handler_result)?);    
        Ok::<_,anyhow::Error>(())
    }).unwrap();

    if runtime.is_job_pending() {
        runtime
            .execute_pending_job()
            .map_err(|e| anyhow::anyhow!("failed to execute pending_job: {:?}", e))
            .unwrap();
    }


    Ok(())
}

async fn async_run_js()->anyhow::Result<()>{
    let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;
    let resolver = (
        BuiltinResolver::default()
            .with_module("path")
            .with_module("url")
            .with_module("os")
            .with_module("main")
            .with_module("dayjs"),
        FileResolver::default()
            .with_path("./")
            .with_path("../../target/debug")
            .with_native(),
    );
    

    let loader = (
        BuiltinLoader::default().with_module("dayjs", include_str!("../js/dayjs.mjs")),
        ModuleLoader::default()
        .with_module("path", PathModule)
        .with_module("url", UrlModule)
        .with_module("os", OsModule),
        ScriptLoader::default(),
        // dyn-load
        // NativeLoader::default(), 
    );

    runtime.set_loader(resolver, loader).await;

    let code = r#"
    import dayjs from 'dayjs'
    import {version,arch,platform} from 'os'
    import url from 'url'
    import {dirname,basename} from 'path'    
    export const handler = async () => {
        //out> Darwin Kernel Version 24.1.0: Thu Oct 10 21:02:27 PDT 2024; root:xnu-11215.41.3~2/RELEASE_X86_64
        print(version())
        //out> x64
        print(arch())
            
        //out> darwin
        print(platform())

        print(dayjs().format())

        print(typeof url)
        await delay(1000)
        //out>  workoss
        print(basename('/home/workoss/'))
        return dayjs().format();
    };
    "#;
    // async_with!(&context => |ctx|{
    //     // 声明模块    
    //     if let Err(e) = Module::declare(ctx.clone(), "main", code).catch(&ctx){
    //         eprintln!("module declare error:{e:#?}");
    //     }
    // }).await;

    let _result = async_with!(&context => |ctx| {

        let global = ctx.globals();

        url::init(&ctx).unwrap();

        global.set("print",Func::from(|s|print(s))).unwrap();
        global.set("delay", Func::from(|millsecond|Promised(delay(millsecond)))).unwrap();

        if let Err(e) = Module::declare(ctx.clone(), "main", code).catch(&ctx){
            eprintln!("module declare error:{e:#?}");
        }
       
        // Module::declare(ctx.clone(), "main", code)
        // .catch(&ctx)
        // .map_err(|e|anyhow::anyhow!("declare catch1:{:#?}",e))
        // .unwrap();
        //获取 handler function
        let m: rquickjs::Object = Module::import(&ctx, "main")
        .catch(&ctx)
        .map_err(|e|anyhow::anyhow!("import catch1:{:#?}",e))?
        .into_future()
        .await
        .catch(&ctx)
        .map_err(|e|anyhow::anyhow!("import catch2:{:#?}",e))?;

        let handler: rquickjs::Function = m.get("handler").catch(&ctx).map_err(|e|anyhow::anyhow!("get handler:{:#?}",e))?;
        // 执行 handler
        let handler_promise: MaybePromise = handler.call(()).catch(&ctx).map_err(|e|anyhow::anyhow!("call handler:{:#?}",e))?;
        let handler_result = handler_promise.into_future::<Value>().await.catch(&ctx).map_err(|e|anyhow::anyhow!("call result:{:#?}",e))?;
        println!("handler_result:{:#?}",val_to_string(&ctx, handler_result)?);    
        Ok::<_,anyhow::Error>(())
    }).await.unwrap();

    if runtime.is_job_pending().await {
        runtime
            .execute_pending_job()
            .await
            .map_err(|e| anyhow::anyhow!("failed to execute pending_job: {:?}", e))
            .unwrap();
    }

    runtime.idle().await;


    Ok(())
}

pub fn from_js_error(ctx: Ctx<'_>, e: rquickjs::Error) -> anyhow::Error {
    if e.is_exception() {
        let val = ctx.catch();

        if let Some(exception) = val.clone().into_exception() {
            anyhow::anyhow!("{exception}")
        } else {
            anyhow::anyhow!(
                val_to_string(&ctx, val).unwrap_or_else(|_| "Internal error".to_string())
            )
        }
    } else {
        Into::into(e)
    }
}

/// Converts an [`anyhow::Error`]  to a [`JSError`].
///
/// If the error is an [`anyhow::Error`] this function will construct and throw
/// a JS [`Exception`] in order to construct the [`JSError`].
pub fn to_js_error(cx: Ctx, e: anyhow::Error) -> rquickjs::Error {
    match e.downcast::<rquickjs::Error>() {
        Ok(e) => e,
        Err(e) => {
            // Ref: https://github.com/sfackler/serde-transcode/issues/8
            if e.to_string()
                .contains("JSError: Exception generated by QuickJS")
            {
                return rquickjs::Error::Exception;
            }

            cx.throw(Value::from_exception(
                Exception::from_message(cx.clone(), &e.to_string())
                    .expect("creating an exception to succeed"),
            ))
        }
    }
}

pub fn val_to_string<'js>(this: &Ctx<'js>, val: Value<'js>) -> anyhow::Result<String> {
    if let Some(symbol) = val.as_symbol() {
        if let Some(description) = symbol.description()?.into_string() {
            let description = description
                .to_string()
                .unwrap_or_else(|e| to_string_lossy(this, &description, e));
            Ok(format!("Symbol({description})"))
        } else {
            Ok("Symbol()".into())
        }
    } else {
        let stringified =
            <convert::Coerced<rquickjs::String>>::from_js(this, val).map(|string| {
                string
                    .to_string()
                    .unwrap_or_else(|e| to_string_lossy(this, &string.0, e))
            })?;
        Ok(stringified)
    }
}

pub fn to_string_lossy<'js>(
    cx: &Ctx<'js>,
    string: &rquickjs::String<'js>,
    error: rquickjs::Error,
) -> String {
    let mut len: qjs::size_t = 0;
    let ptr = unsafe { qjs::JS_ToCStringLen2(cx.as_raw().as_ptr(), &mut len, string.as_raw(), 0) };
    let buffer = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };

    // The error here *must* be a Utf8 error; the `JSString::to_string()` may
    // return `JSError::Unknown`, but at that point, something else has gone
    // wrong too.

    let mut utf8_error = match error {
        rquickjs::Error::Utf8(e) => e,
        _ => unreachable!("expected Utf8 error"),
    };
    let mut res = String::new();
    let mut buffer = buffer;
    loop {
        let (valid, after_valid) = buffer.split_at(utf8_error.valid_up_to());
        res.push_str(unsafe { str::from_utf8_unchecked(valid) });
        res.push(char::REPLACEMENT_CHARACTER);

        let lone_surrogate = matches!(after_valid, [0xED, 0xA0..=0xBF, 0x80..=0xBF, ..]);

        let error_len = if lone_surrogate {
            3
        } else {
            utf8_error
                .error_len()
                .expect("Error length should always be available on underlying buffer")
        };

        buffer = &after_valid[error_len..];
        match str::from_utf8(buffer) {
            Ok(valid) => {
                res.push_str(valid);
                break;
            }
            Err(e) => utf8_error = e,
        }
    }
    res
}

pub struct ModuleEvaluator;

impl ModuleEvaluator {
    pub async fn eval_js<'js>(
        ctx: Ctx<'js>,
        name: &str,
        source: &str,
    ) -> rquickjs::Result<Module<'js, Evaluated>> {
        let (module, module_eval) = Module::declare(ctx, name, source)?.eval()?;
        module_eval.into_future::<()>().await?;
        Ok(module)
    }

    pub async fn eval_rust<'js, M>(
        ctx: Ctx<'js>,
        name: &str,
    ) -> rquickjs::Result<Module<'js, Evaluated>>
    where
        M: ModuleDef,
    {
        let (module, module_eval) = Module::evaluate_def::<M, _>(ctx, name)?;
        module_eval.into_future::<()>().await?;
        Ok(module)
    }
}

#[cfg(test)]
mod tests {
    use llrt_modules::{
        os::OsModule,
        path::PathModule,
        url::{self, UrlModule},
    };
    use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Module, async_with};
    use rquickjs::{Value, promise::MaybePromise};
    use rquickjs::{
        loader::{
            BuiltinLoader, BuiltinResolver, FileResolver, ModuleLoader, ScriptLoader,
        },
        prelude::Func,
    };

    use crate::val_to_string;

    fn print(msg: String) {
        println!("{msg}");
    }

    #[cfg(feature = "tokio-async")]
    #[tokio::test]
    async fn test_handler() -> anyhow::Result<()> {
        let runtime = AsyncRuntime::new()?;
        let context = AsyncContext::full(&runtime).await?;
        let resolver = (
            BuiltinResolver::default()
                .with_module("path")
                .with_module("url")
                .with_module("os")
                .with_module("main")
                .with_module("dayjs"),
            FileResolver::default()
                .with_path("./")
                .with_path("../../target/debug")
                .with_native(),
        );
        let module_loader = ModuleLoader::default()
            .with_module("path", PathModule)
            .with_module("url", UrlModule)
            .with_module("os", OsModule);

        let loader = (
            BuiltinLoader::default().with_module("dayjs", include_str!("../js/dayjs.mjs")),
            module_loader,
            ScriptLoader::default(),
            NativeLoader::default(),
        );

        runtime.set_loader(resolver, loader).await;

        let code = r#"
        import dayjs from 'dayjs';    
        export const handler = async () => {
            print('----------');
            return dayjs().format();
        };
        "#;
        // async_with!(&context => |ctx|{
        //     // 声明模块    
        //     if let Err(e) = Module::declare(ctx.clone(), "main", code).catch(&ctx){
        //         eprintln!("module declare error:{e:#?}");
        //     }
        // }).await;

        let _result = async_with!(&context => |ctx| {

            let global = ctx.globals();

            url::init(&ctx).unwrap();

            global.set("print",Func::from(|s|print(s))).unwrap();

            if let Err(e) = Module::declare(ctx.clone(), "main", code).catch(&ctx){
                eprintln!("module declare error:{e:#?}");
            }
           
            // Module::declare(ctx.clone(), "main", code)
            // .catch(&ctx)
            // .map_err(|e|anyhow::anyhow!("declare catch1:{:#?}",e))
            // .unwrap();
            //获取 handler function
            let m: rquickjs::Object = Module::import(&ctx, "main")
            .catch(&ctx)
            .map_err(|e|anyhow::anyhow!("import catch1:{:#?}",e))?
            .into_future()
            .await
            .catch(&ctx)
            .map_err(|e|anyhow::anyhow!("import catch2:{:#?}",e))?;

            let handler: rquickjs::Function = m.get("handler").catch(&ctx).map_err(|e|anyhow::anyhow!("get handler:{:#?}",e))?;
            // 执行 handler
            let handler_promise: MaybePromise = handler.call(()).catch(&ctx).map_err(|e|anyhow::anyhow!("call handler:{:#?}",e))?;
            let handler_result: Value<'_> = handler_promise.into_future::<Value>().await.catch(&ctx).map_err(|e|anyhow::anyhow!("call result:{:#?}",e))?;
            println!("handler_result:{:#?}",val_to_string(&ctx, handler_result));    
            Ok::<_,anyhow::Error>(())
        }).await.unwrap();

        if runtime.is_job_pending().await {
            runtime
                .execute_pending_job()
                .await
                .map_err(|e| anyhow::anyhow!("failed to execute pending_job: {:?}", e))
                .unwrap();
        }

        runtime.idle().await;

        Ok(())
    }

    #[cfg(feature = "tokio-async")]
#[tokio::test]
async fn test_01() -> anyhow::Result<()> {
    let runtime = AsyncRuntime::new()?;
    let context = AsyncContext::full(&runtime).await?;

    let resolver = (
        BuiltinResolver::default()
            .with_module("path")
            .with_module("url")
            .with_module("os")
            .with_module("dayjs"),
        FileResolver::default()
            .with_path("./")
            .with_path("../../target/debug")
            .with_native(),
    );

    let loader = (
        BuiltinLoader::default().with_module("dayjs", include_str!("../js/dayjs.mjs")),
        ModuleLoader::default()
            .with_module("path", PathModule)
            .with_module("url", UrlModule)
            .with_module("os", OsModule),
        ScriptLoader::default(),
    );

    runtime.set_loader(resolver, loader).await;

    let code = r#"
            // const {a} = await import('index.js);
            // await import('index.js');
            import {version,arch,platform} from 'os'
            import url from 'url'
            import {dirname,basename} from 'path'
            import dayjs from 'dayjs'
            const handler = async () => {
                //out> Darwin Kernel Version 24.1.0: Thu Oct 10 21:02:27 PDT 2024; root:xnu-11215.41.3~2/RELEASE_X86_64
                print(version())
                //out> x64
                print(arch())
                
                //out> darwin
                print(platform())

                print(dayjs().format())

                print(typeof url)
                await delay(1000)
                //out>  workoss
                print(basename('/home/workoss/'))
                // throw new Error('---');
            }
            
            handler();     
           
        "#;

    // ModuleEvaluator::eval_rust::<OsModule>(ctx.clone(), "os")
    //     .await
    //     .unwrap();

    async_with!(context => |ctx| {
    //   import { arch,version } from "os";
        let global = ctx.globals();

        url::init(&ctx).unwrap();

        global.set("print",Func::from(|s|print(s))).unwrap();
        global.set("delay", Func::from(|millsecond|Promised(delay(millsecond)))).unwrap();

        // if let Err(CaughtError::Value(err)) = ctx.eval::<(),_>(code).catch(&ctx) {
        //     println!("eval: {:#?}",err);
        // }

    //     let promises = Module::evaluate(ctx.clone(), "main", code)
    //     .catch(&ctx)
    //     .map_err(|e| anyhow::anyhow!("failed to evaluate user script: {:?}", e))
    //     .unwrap();
    // promises.into_future::<()>()
    // .await
    // .catch(&ctx)
    // .map_err(|e|anyhow::anyhow!("failed to wait for user script to finish: {:?}", e))
    // .unwrap();




       let md_rs =   Module::evaluate(ctx.clone(), "ext.js", code)
        .catch(&ctx)
        .map_err(|e|{
            println!("ex:{:#?}",e);
            if e.is_exception() {
                let val = ctx.catch();
                if let Some(exception) = val.clone().into_exception() {
                    anyhow::anyhow!("{exception}")
                } else {
                    anyhow::anyhow!("Internal error")
                }
            }else {
                anyhow::anyhow!("other error:{e:#?}")
            }
        })
        .unwrap();

        let rs = md_rs.finish::<Value>()
        .catch(&ctx)
        .map_err(|e|anyhow::anyhow!("failed to wait for user script to finish: {:?}", e))
        .unwrap();
        println!("rs:{:#?}",rs);

        // .finish::<Value>()
        // .unwrap();
    })
    .await;

    if runtime.is_job_pending().await {
        runtime
            .execute_pending_job()
            .await
            .map_err(|e| anyhow::anyhow!("failed to execute pending_job: {:?}", e))
            .unwrap();
    }

    runtime.idle().await;

    Ok(())
}

}

