(:def
  (fib (n)
      (:ifelse
        (n < 2)
        (1)
        (+ (fib (- n 1)) (fib (- n 2))))
  )
)
