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

use crate::heap::desc::DescriptorParser;

na!(
    JLSReg,
    "java/lang/System",
    "registerNatives",
    "()V",
    th,
    f,
    {
        reg!(th.registry, N0, N1, VM);
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
                true,
            )
        }
        f.push_obj(this);
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

na!(
    N1,
    "java/lang/System",
    "arraycopy",
    "(Ljava/lang/Object;ILjava/lang/Object;II)V",
    th,
    f,
    {
        use crate::heap::class::Object;
        use rp::Rp;
        let src: Rp<Object> = (f.local_vars()[0] as usize).into();
        let src_p = f.local_vars()[1] as usize;
        let mut dest: Rp<Object> = (f.local_vars()[2] as usize).into();
        let dest_p = f.local_vars()[3] as usize;
        let len = f.local_vars()[4] as usize;

        println!(
            "system array copy start {} {} {} {}",
            src.class.name, src_p, dest_p, len
        );
        let mut parser = DescriptorParser::new(src.class.name.as_bytes());
        let d = parser.parse_arr();
        let mut sz = match &src.class.element_class.name[0..1] {
            "B" | "Z " => 1,
            "C" | "S" => 2,
            "F" | "I" => 4,
            _ => 8,
        };

        if d.0 != 1 {
            sz = 8;
        }

        let total_size = sz * len;

        // copy byte by byte
        for i in 0..total_size {
            let x: u8 = src.get(src_p * sz + i);
            dest.set(dest_p * sz + i, x);
        }

        println!("system array copy end");
    }
);
