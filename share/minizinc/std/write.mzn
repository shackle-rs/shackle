include "fzn_write.mzn";
include "fzn_write_reif.mzn";

/** @group globals.array
    Creates a new array \a O from an input array \a I with a change at position \a i to take value \a v

    \a I is an array of integers
    \a O is an array of integers with same index set as \a I
    \a i is an index for \a I
    \a v is an integer value 
*/
predicate write(array[$$E] of var int: I, var $$E: i, var int: v,
                array[$$E] of var int: O) =
    assert(index_set(O) = index_set(I),"writet: index set of I must be same as O") /\
    fzn_write(erase_enum(I), erase_enum(i), erase_enum(v), erase_enum(O));

/** @group globals.array
    Creates a new array \a O from an input array \a I with a change at position \a i to take value \a v

    \a I is an array of integers
    \a i is an index for \a I
    \a v is an integer value 
*/
function array[$$E] of var int: write(array[$$E] of var int: I, var $$E: i, var int: v) =
    let { array[index_set(I)] of var int: O;
	      constraint fzn_write(erase_enum(I), erase_enum(i), erase_enum(v), erase_enum(O));
    } in O;

%-----------------------------------------------------------------------------%
