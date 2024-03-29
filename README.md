![](/icon/Maxwell_316x316.png)
# Maxwell Chess Engine
 A Chess engine written from scratch in Rust.<br>
 If you use this code verbatim, or as a reference, please credit me!<br><br>
 
 Rating Lists featuring Maxwell: [CCRL Blitz](https://computerchess.org.uk/ccrl/404/) | [MCERL](https://www.chessengeria.eu/mcerl)<br>
 [Play against Maxwell on Lichess!](https://lichess.org/@/MaxwellOnLC) | [Some of Maxwell's Games](https://www.chess.com/library/collections/maxwells-games-my-chess-engine-2FFU82NM4)

## Rough Roadmap
 - Tweak search values & thresholds
 - Internal Iterative Deepening
 - Static Exchange Evaluation
 - Late Move Pruning
 - Write an NNUE implementation! I've learned how Neural Networks work, so I'm really excited to get started on that
 - Multithreading

## Features
#### Parameters
 - fen=\<FEN STRING>: Sets up the board by a fen string (Doesn't work for UCI games) (default=STARTING_FEN)
 - debug_output=\<BOOLEAN>: Toggle debug output that gets outputed per ply (default=true)
 - opening_book=\<BOOLEAN>: Toggle built-in opening book (default=false)
 - time_management=\<BOOLEAN>: Toggle time management, if false the bot will use all the remaining time (default=true)
 - hash_size=\<INTEGER>: Sets the hash size in Megabytes, there's also a UCI option for this under the name "Hash" (default=256)
#### UCI Interface
 - uci, isready, ucinewgame, position, go, stop, and quit commands
 - "position" is only implemented for "position startpos", "position fen" is not yet implemented
 - "Hash" UCI option, which sets the hash / transposition table size in Megabytes
#### Board Representation
 - Purely bitboards
 - Supports loading from FEN strings
#### Move Generation
 - Basic handcrafted opening book
 - Magic bitboards for sliding pieces
 - Hardcoded pawn movement
 - Bitboard masks for other pieces calculated at startup
 - Calculates pseudo-legal moves, then skips illegal moves in move loop
#### Evaluation
 - Material count
 - Piece square tables
   - Separate middlegame and endgame tables for pawns and kings
 - Passed, isolated and doubled pawns
 - Attacked squares around kings
#### Move Ordering
 - Best move from the previous iteration, otherwise whatever move from the transposition table
 - MVV-LVA
 - 2 Killer Moves
 - History Heuristic
   - Indexed by side to move, move start square, move end square
#### Search
 - Single Threaded ~ For now ;)
 - Negamax
 - Iterative Deepening
 - Alpha-Beta Pruning
 - Late Move Reductions
 - Principal Variation Search
 - Reverse Futility Pruning (Static Null Move Pruning)
 - Null Move Pruning
 - Razoring
 - Internal Iterative Reductions
 - Quiescence Search
   - Delta Pruning
   - No TT Lookups
 - Transposition Table
   - UCI "Hash" option to change max size, default is 256 MB
   - Replacement scheme prefers higher depth and exact evaluation bound
 - Search Extensions
   - Checks
   - Pawn moves to the 2nd or 7th rank
 - Time management
   - If less than 7 moves have been played, it uses 2.5% of it's remaining time, otherwise 7%
   - This value is then also clamped between 0.05 and 30.0 seconds

## Helpful Sources & References
 #### Thanks to Sebastian Lague for making his YouTube series, which inspired me to make my own engine!
 - [Sebastian Lague's Chess Programming series](https://www.youtube.com/playlist?list=PLFt_AvWsXl0cvHyu32ajwh2qU1i6hl77c)

 #### When I'm not sure what to do next, I like to read through other engine's code for ideas. <br> I try not to copy line for line, but in any case here are the engine's I've referenced:
 - [Boychesser](https://github.com/analog-hors/Boychesser/)
 - [Weiawaga](https://github.com/Heiaha/Weiawaga/)
 - [Rustic (Engine and Book)](https://github.com/mvanthoor/rustic)
 - [Lynx](https://github.com/lynx-chess/Lynx/)
 - [Fruit 2.1](https://github.com/Warpten/Fruit-2.1/)
 - [Tcheran](https://github.com/jgilchrist/tcheran/)
 - [MadChess](https://github.com/ekmadsen/MadChess/)
 - [Black Marlin](https://github.com/jnlt3/blackmarlin/)
 - [Ethereal](https://github.com/AndyGrant/Ethereal/)

 #### And some other helpful resources
 - [The Chess Programming Wiki](https://www.chessprogramming.org/Main_Page)
 - [BBC Engine Development](https://www.youtube.com/playlist?list=PLmN0neTso3Jxh8ZIylk74JpwfiWNI76Cs)
 - [Perfect 2021 Opening Book](https://sites.google.com/site/computerschess/perfect-2021-books)
 - [Cute Chess](https://cutechess.com/)
 - [PVS Implementation](https://web.archive.org/web/20071030220825/http://www.brucemo.com/compchess/programming/pvs.htm)
 - [LMR Implementation](https://web.archive.org/web/20150212051846/http://www.glaurungchess.com/lmr.html)
 - [Mediocre Chess](https://mediocrechess.blogspot.com/)
 - [Chess Programming Reddit](https://www.reddit.com/r/chessprogramming/)
 - [TalkChess Forum](https://talkchess.com/forum3/index.php)
 - [Stockfish Features List](https://www.chessprogramming.org/Stockfish#Search)