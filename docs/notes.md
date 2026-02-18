# Data structure

- Stack and heap are part of memeory that are available to our code to use at runtime
- All the data stored in the stack is of fixed size.
- The data that is of unknow size or whose size might change is stored in heap.

## Ownership

- Keep track of what part of code uses what part of the data on heap, minimizing the amount of duplicate data on the heap and cleaning up the unused data on the heap.
- We can borrow values using & which is a reference i.e pointer or address to the original value. If we want to mutate the value in the reference place the variable should be mutable. If we have mutable reference to a variable it cannot have another reference. See the following example. This actuall help solving the race condition at the compile time itself.

```rust
fn main(){
    let mut s = String::from("Vedang");

    let r1 = &mut s;
    println!("{r1} {r2}");
    change(&mut s);

}

fn change(s: &mut String){
    s.push_str(" Barhate");
}
```

- We can have multiple immutable references in same scope as we are reading the data and not manupliating it, but if we have mutable and immutable refrences in same scope it might error out and it depends on the usage of the refrences. If we consume the imutable references first or the mutable reference first and is not used again the same scope we can keep both mutable and immutable references
  - At any given time, you can have either one mutable reference or any number of immutable references.
  - References must always be valid.
  - **Reference scope start from where it is introduced and continues through last time that reference is used**

NOT VALID

```rust
let mut s = String::from("hello");

let r1 = &s; // no problem
let r2 = &s; // no problem
let r3 = &mut s; // BIG PROBLEM

println!("{r1}, {r2}, and {r3}"); // Here are consuming all three

```

VALID

```rust
let mut s = String::from("Vedang");

let r3 = &mut s;
println!("{r3}");
let r1 = &s;
let r2 = &s;

println!("{r1} {r2}");
```
