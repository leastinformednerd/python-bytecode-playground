A core component of the compiler's broad strokes bytecode emitting strategy is the idea of a block. In pseudo-Haskell a block is defined recursively in something like
```Haskell
data Block = Block Instruction (Maybe Block)
```
where `Instruction` represents a CPython operator and argument

We can do some very nice things with these blocks if we track some additional information alongside them
```Haskell
data BlockState = BlockState { inner :: Block; height :: Int; depth :: Int }
```
where we use height to track the effect on the size of the stack (i.e POP_TOP has a height of -1, LOAD_CONST has 1, and BINARY_OP has -1, {-2 +1}) and use depth to track the greatest depth the block digs into the existing stack (for the previous instructions that's -1, 1 and -2 respectively). This composition is defined by something like this
```Haskell
composeBlocks :: Block -> Block -> Block

composeBlockState a b = BlockState
	(composeBlocks $ inner a $ inner b)
	(height a + height b)
	(min (depth b) (height b + depth a))
```

`BlockState`s that fulfill the condition that `height == 0 && depth >= 0` can be composed easily, since the state of the stack before and after is identical. This is very useful, for instance all functions satisfy this (kind of? the return value is placed on the stack, but returned with RETURN_VALUE, and I'm not sure whether to count it) and all mutation of the  non-stack state (i.e I/O, storing into a variable) meets this condition. Somewhat interesting the program itself doesn't necessarily have to meet this condition, it simply needs a depth >= 0. 

Unfortunately not all blocks satisfy these conditions operation (trivially, almost every (possibly just every non-no-op operation, I haven't checked) touches the stack) so care needs to be taken to ensure those meet all the conditions they're expected to (I imagine I'll be writing a lot of assertions)