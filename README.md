# Chress

A UCI compliant chess move generator.

## Technical Specs

- Bitboard board representation
- Magic bitboards for sliding moves
- Piece-lookup tables for knights/kings
- Bulk pawn move generation
- Compressed Move + Flags structs

## Usage (Debugging CLI)

1. Clone the project locally

```sh
git clone https://github.com/LucDeCaf/chress
```

2. CD into the project folder and run the CLI using Cargo

```sh
cd chress
cargo run -p chress_cli
```

3. Enter the 'disp' command into the terminal and press Enter to confirm the program is working

```sh
<< disp
>> 8  R N B Q K B N R
>> 7  P P P P P P P P
>> 6  . . . . . . . .
>> 5  . . . . . . . .
>> 4  . . . . . . . .
>> 3  . . . . . . . .
>> 2  p p p p p p p p
>> 1  r n b q k b n r
>>    A B C D E F G H
```

NB: See the README.md in chress-cli for a full list of debugging commands

## Usage (UCI)

1. Clone the project locally

```sh
git clone https://github.com/LucDeCaf/chress
```

2. CD into the project folder and run the CLI using Cargo

```sh
cd chress
cargo run -p chress_cli
```

3. Enter the 'uci' command to enter UCI mode

```sh
<< uci
>> id name Chress
>> id author Luc de Cafmeyer
>> uciok
```
