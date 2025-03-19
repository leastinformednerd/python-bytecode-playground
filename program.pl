(:def
  (fib (n)
      (:ifelse
        (n < 2)
        (n)
        (+ (fib (- n 1)) (fib (- n 2))))
  )
)
(print (fib (int (input))))
