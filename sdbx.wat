(module
	(import "console" "log" (func $log (param i32)))
	(func $add (param $x i32) (param $y i32) (result i32)
		local.get $x
		local.get $y
		i32.add
	)
	(func $a
		i32.const 1
		i32.const 5
		call $add
		call $log
	)
	(table $a 1 funcref)
	(func $main

	)
	(start $a)

	;; (export "add" (func $add))
)