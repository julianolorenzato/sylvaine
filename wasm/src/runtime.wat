(module
    (type $Closure
        (struct
            (field $code (ref func))
            (field $env (ref any))
        )
    )

    ;; EXAMPLE OF LAMBDA LIFT WITH CLOSURE CONVERSION
    (type ;0; (func (param $a (ref $Lisp_obj))))

    (type $point (struct (field i32) (field i32)))
    (func (export "p") (param $d i32)
        local.get $d
        (i32.const 12)
        (i32.add)
        drop
    )
)