# RRR

The data structure `RRR` is a *static* bit vector that answers rank queries
in O(1), and provides implicit compression. It is known as a succinct data
structure, which means that even though it is compressed, we can operate on it
efficiently without decompressing the whole bit vector.

To construct a RRR sequence, divides (logically) our bit vector into `block`s of
`b` bits, and each block is said to be of `class[c]`, where `class[c]` is the
sorted list of bit patterns which has the number of set bits `c`.

If `b` is 3:
- `class[0]`: `[000]`
- `class[1]`: `[001, 010, 100]`
- `class[2]`: `[011, 101, 110]`
- `class[3]`: `[111]`

If `b` is 4:
- `class[0]`: `[0000]`
- `class[1]`: `[0001, 0010, 0100, 1000]`
- `class[2]`: `[0011, 0101, 0110, 1001, 1010, 1100]`
- `class[3]`: `[0111, 1011, 1101, 1110]`
- `class[4]`: `[1111]`

We can describe each `block[i]` using a pair of `c` and `o`, where `o` is
the index of each `class[c]`. For example, when `b` is 4 and `c` is 1,
assign 0 to `0001`, 1 to `0010`, 2 to `0100` and 3 to `1000`.

Note that `class[c]` contains offsets for the number of `comb(b, c)`, and
the bit patterns for each offset follow a specific order. Offsets starting with 0,
`comb(b-1, c)`, exist and precede blocks starting with 1, `comb(b-1, c-1)`.
By utilizing this rule during encoding/decoding, the bit patterns can be reconstructed
efficiently from the two values `c` and `o`.
