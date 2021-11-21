macro_rules! map {
    {
        $($key: expr => $value: expr),+
    }  => {
       {
           let mut m = std::collections::HashMap::new();
           $(
            m.insert($key.to_string(), $value.to_string());
           )+
           m
       }
    };
}
use std::collections::HashMap;

na!(
    JLSReg,
    "java/lang/System",
    "registerNatives",
    "()V",
    th,
    f,
    {
        reg!(f.registry, N0, VM);
    }
);

na!(
    N0,
    "java/lang/System",
    "initProperties",
    "(Ljava/util/Properties;)Ljava/util/Properties;",
    th,
    f,
    {
        // call Properties.set()
        let this = f.this();

        let props = sys_props();
        for (k, v) in props.iter() {
            let args = [
                this.ptr() as u64,
                th.heap.new_jstr(k).ptr() as u64,
                th.heap.new_jstr(v).ptr() as u64,
            ];
            th.invoke_obj(
                this.get_mut(),
                "setProperty",
                "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;",
                &args,
            )
        }
        f.stack.push_obj(this);
    }
);

fn sys_props() -> HashMap<String, String> {
    map! {
        "java.version"=>         "1.8.0",
        "java.vendor" =>          "rust-jvm",
        "java.vendor.url"=>      "https://github.com/Salpadding/jvm-rust",
        "java.home"=>            "todo",
        "java.class.version"=>   "52.0",
        "java.class.path"=>      "todo",
        "java.awt.graphicsenv"=> "sun.awt.CGraphicsEnvironment",
        "os.name"=>              "linux",   // todo
        "os.arch"=>              "x64", // todo
        "os.version"=>           "",             // todo
        "file.separator"=>       "/",            // todo os.PathSeparator
        "path.separator"=>       ":",            // todo os.PathListSeparator
        "line.separator"=>       "\n",           // todo
        "user.name"=>            "",             // todo
        "user.home"=>            "",             // todo
        "user.dir"=>             ".",            // todo
        "user.country"=>         "CN",           // todo
        "file.encoding"=>        "UTF-8",
        "sun.stdout.encoding"=>  "UTF-8",
        "sun.stderr.encoding"=>  "UTF-8"
    }
}

na!(VM, "sun/misc/VM", "initialize", "()V", th, f, {
    // call initializeSystemClass
    let m = f.heap.loader.load("java/lang/System");
    let m = m.lookup_method_in_class("initializeSystemClass", "()V");
    th.push_frame(th.new_frame(m));
});
