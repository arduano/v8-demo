use std::sync::Once;

const JS_CODE: &'static str = "
export function add(a, b) {
    return a + b;
}
";

fn main() {
    initialize_v8();

    // Declare the V8 execution context
    let isolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    let executed = exec_module(JS_CODE, scope);

    let function_name = v8::String::new(scope, "add").unwrap();
    let function: v8::Local<v8::Function> = executed
        .get(scope, function_name.into())
        .expect("Failed to get the function.")
        .try_into()
        .expect("Failed to cast to a function");

    println!("Extracted function:");
    println!("{}", function.to_rust_string_lossy(scope));

    // Make args
    let args = vec![
        v8::Integer::new(scope, 1).into(),
        v8::Integer::new(scope, 2).into(),
    ];

    // Call the function
    let recv = v8::undefined(scope).into();
    let result = function
        .call(scope, recv, &args)
        .expect("Failed to convert to function.");

    println!("Result:");
    println!("{}", result.to_rust_string_lossy(scope));
}

fn exec_module<'a>(code: &str, scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Object> {
    let source = to_v8_source(scope, code, "<eval string>");
    let module =
        v8::script_compiler::compile_module(scope, source).expect("Failed to compile the module.");

    module
        .instantiate_module(scope, resolve_module_callback)
        .expect("Instantiation failure.");

    module.evaluate(scope).expect("Evaluation failure.");

    let obj = module
        .get_module_namespace()
        .to_object(scope)
        .expect("Failed to get the module namespace.");

    obj
}

static INIT_V8: Once = Once::new();
fn initialize_v8() {
    INIT_V8.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

fn resolve_module_callback<'a>(
    _: v8::Local<'a, v8::Context>,
    _: v8::Local<'a, v8::String>,
    _: v8::Local<'a, v8::FixedArray>,
    _: v8::Local<'a, v8::Module>,
) -> Option<v8::Local<'a, v8::Module>> {
    panic!("Module resolution not supported.")
}

fn new_script_origin<'s>(
    scope: &mut v8::HandleScope<'s>,
    resource_name: &str,
    source_map_url: &str,
) -> v8::ScriptOrigin<'s> {
    let resource_name_v8_str = v8::String::new(scope, resource_name).unwrap();
    let resource_line_offset = 0;
    let resource_column_offset = 0;
    let resource_is_shared_cross_origin = true;
    let script_id = 123;
    let source_map_url = v8::String::new(scope, source_map_url).unwrap();
    let resource_is_opaque = false;
    let is_wasm = false;
    let is_module = true;
    v8::ScriptOrigin::new(
        scope,
        resource_name_v8_str.into(),
        resource_line_offset,
        resource_column_offset,
        resource_is_shared_cross_origin,
        script_id,
        source_map_url.into(),
        resource_is_opaque,
        is_wasm,
        is_module,
    )
}

fn to_v8_source(
    scope: &mut v8::HandleScope,
    js_code: &str,
    source_path: &str,
) -> v8::script_compiler::Source {
    let code = v8::String::new(scope, js_code).unwrap();
    let origin = new_script_origin(scope, source_path, &format!("file://{source_path}.map"));
    v8::script_compiler::Source::new(code, Some(&origin))
}
