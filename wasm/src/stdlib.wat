(module
    ;; (func $+ (export "stdlib" "+") (param $lisp_obj))

    (type $Lisp_obj
        (struct
            (field i32) ;; Type 0=Int, 1=Symbol, 2=List, etc...
            (field anyref)
        )
    )

    (type $Cons_cell
        (struct
            (field (mut anyref)) ;; CAR
            (field (mut anyref)) ;; CDR
        )
    )

    (func $cons (param $car anyref) (param $cdr anyref) (result anyref)
        (struct.new $Cons_cell
            (local.get $car)
            (local.get $cdr)
        )
    )

    (func $car (param $cell anyref) (result anyref)
        ;; 1. Converte a referência genérica (anyref) para o tipo da nossa struct
        ;; O comando ref.cast garante que o programa falhe se o objeto não for uma lista
        (local $typed_cell (ref $Cons_cell))
        (local.set $typed_cell (ref.cast (ref $Cons_cell) (local.get $cell)))

        ;; 2. Extrai o valor do primeiro campo (CAR)
        (struct.get $Cons_cell 0 (local.get $typed_cell))
    )

    (func $cdr (export "stdlib" "cdr") (param $cell anyref) (result anyref)
        (local $typed_cell (ref $Cons_cell))
        (local.set $typed_cell (ref.cast (ref $Cons_cell) (local.get $cell)))

        ;; 2. Extrai o valor do segundo campo (CDR)
        (struct.get $Cons_cell 1 (local.get $typed_cell))
    )
)