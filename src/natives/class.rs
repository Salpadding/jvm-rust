use crate::heap::class::Class;

na!(
    ClassReg,
    "java/lang/Class",
    "registerNatives",
    "()V",
    th,
    f,
    { reg!(th.registry, N0, N1, N2, N3, N4) }
);

na!(
    N0,
    "java/lang/Class",
    "getPrimitiveClass",
    "(Ljava/lang/String;)Ljava/lang/Class;",
    th,
    f,
    {
        let s = f.this().jstring();
        let mut cl = f.heap.loader.load(&s);

        if cl.clinit(th) {
            th.revert_pc();
            return;
        }

        f.stack.push_obj(cl.j_class)
    }
);

na!(
    N1,
    "java/lang/Class",
    "desiredAssertionStatus0",
    "(Ljava/lang/Class;)Z",
    th,
    f,
    { f.stack.push_u32(0) }
);

na!(
    N2,
    "java/lang/Class",
    "getName0",
    "()Ljava/lang/String;",
    th,
    f,
    {
        let cl = f.this().class;
        let js = f.heap.new_jstr(&cl.name);
        f.stack.push_obj(js)
    }
);

na!(
    N3,
    "java/lang/Class",
    "forName0",
    "(Ljava/lang/String;ZLjava/lang/ClassLoader;Ljava/lang/Class;)Ljava/lang/Class;",
    th,
    f,
    {
        use crate::heap::class::Object;
        use rp::Rp;
        let js: Rp<Object> = (f.local_vars[0] as usize).into();
        let n = js.jstring();
        let cl = f.heap.loader.load(&n);
        f.stack.push_obj(cl.j_class)
    }
);

na!(
    N4,
    "java/lang/Class",
    "getDeclaredFields0",
    "(Z)[Ljava/lang/reflect/Field;",
    th,
    f,
    {
        use crate::heap::class::ClassMember;
        use rp::Rp;

        let jclass = f.this();
        let pub_only = f.local_vars[1] != 0;
        let mut class = jclass.extra_class();
        let field_class = f.heap.loader.load("java/lang/reflect/Field");
        let fields: Vec<Rp<ClassMember>> = class
            .fields
            .iter_mut()
            .filter(|x| !pub_only || (x.access_flags.is_public()))
            .map(|x| {
                let y: Rp<ClassMember> = x.into();
                y
            })
            .collect();

        let mut field_arr = f.heap.new_array("java/lang/reflect/Field", fields.len());

        f.stack.push_obj(field_arr);
        if fields.len() == 0 {
            return;
        }

        for i in 0..fields.len() {
            let mut o = Class::new_obj_size(field_class, field_class.ins_fields.len() + 1);
            let p = o.ptr() as u64;

            // call init method of Field Constructor
            o.set_field_ref("clazz", class.j_class);
            println!("field name = {}", &fields[i].name);
            o.set_field_ref("name", f.heap.new_jstr(&fields[i].name));
            o.set_field("slot", fields[i].id as u64);
            o.set_field("modifiers", fields[i].access_flags.0 as u64);
            o.fields()[o.fields().len() - 1] = fields[i].ptr() as u64;
            field_arr.set(i, p);
        }
    }
);
