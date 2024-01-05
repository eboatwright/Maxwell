![](/icon/Maxwell_316x316.png)
# Maxwell Chess Engine
 A Chess engine written from scratch in Rust!<br>
 If you use this code verbatim, or as a reference, please credit me!<br><br>
 [Play against Maxwell on Lichess!](https://lichess.org/@/MaxwellOnLC) | [Some of Maxwell's Games](https://www.chess.com/library/collections/maxwells-games-my-chess-engine-2FFU82NM4)

## Features
#### Parameters
 - fen=\<FEN STRING>: Sets up the board by a fen string (Doesn't work for UCI games) (default=STARTING_FEN)
 - debug=\<BOOLEAN>: Toggle debug output that gets outputed per ply (default=true)
 - opening_book=\<BOOLEAN>: Toggle opening book (default=true)
 - time_management=\<BOOLEAN>: Toggle time management, if false the bot will use all the remaining time (default=true)
#### UCI Interface
 - Only supports games from startpos
 - uci, isready, ucinewgame, position, go, stop, and quit commands
#### Board Representation
 - Purely bitboards
 - Supports loading from FEN strings
#### Move Generation
 - Basic handcrafted opening book
 - Magic bitboards for sliding pieces
 - Hardcoded pawn movement
 - Bitboard masks for other pieces calculated at startup
#### Evaluation
 - Material count
 - Piece square tables
   - Separate middlegame and endgame tables for pawns and kings
 - Passed, isolated and doubled pawns
 - Attacked squares around kings
#### Move Ordering
 - Hash move / best move from previous iteration
 - MVV-LVA
 - 2 Killer moves
 - History heuristic
 - Castling
 - Promotions
 - Penalty for moving a piece to a square an opponent's piece attacks
#### Search
 - Iterative deepening
 - Aspiration windows
   - Starts at 40 and multiplies by 4 if out of alpha beta bounds
 - Time management
   - If less than 7 moves have been played, it uses 2.5% of it's remaining time, otherwise 7%
   - This value is then also clamped between 0.25 and 20.0 seconds
 - Exits search if a mate is found within search depth
 - Alpha beta pruning
 - Quiescence search with Delta Pruning
 - Transposition table
   - No set max size, but entries get removed after 10 moves without hits
 - Null move pruning
 - Razoring
 - Reverse futility pruning
 - Late move reduction
 - Search extensions
   - Promotions
   - Checks

## Helpful Sources
 - [Sebastian Lague's Chess Programming series](https://www.youtube.com/playlist?list=PLFt_AvWsXl0cvHyu32ajwh2qU1i6hl77c)
 - [The Chess Programming Wiki](https://www.chessprogramming.org/Main_Page)
 - [BBC Engine Development](https://www.youtube.com/playlist?list=PLmN0neTso3Jxh8ZIylk74JpwfiWNI76Cs)
 - [Lynx](https://github.com/lynx-chess/Lynx/)
 - [Weiawaga](https://github.com/Heiaha/Weiawaga/)
