![](/icon/Maxwell_316x316.png)
# Maxwell Chess Engine
 A Chess engine written from scratch in Rust!<br>
 If you use this code verbatim, or as a reference, please credit me!<br><br>
 [Play against Maxwell on Lichess!](https://lichess.org/@/MaxwellOnLC) | [Some of Maxwell's Games](https://www.chess.com/library/collections/maxwells-games-my-chess-engine-2FFU82NM4)

## Features
#### UCI Interface
 - Only supports games from startpos
 - uci, isready, ucinewgame, position, go, stop, and quit commands
#### Board Representation
 - Purely bitboards
 - Supports loading from FEN strings
#### Move Generation
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
 - Capturing a piece of higher value
 - 2 Killer moves
 - History heuristic
 - Castling
 - Promotions
 - Penalty for moving a piece to a square an opponent's piece attacks
#### Search
 - Iterative deepening
 - Aspiration windows
   - Starts at 40 and multiplies by 4 if out of alpha beta bounds
 - Time management: if less than 7 moves have been played, it uses 2.5% of it's remaining time, otherwise 7%
 - Exits search if a mate is found within search depth
 - Alpha beta pruning
 - Quiescence search
 - Transposition table: No set max size, but moves get removed after 10 moves
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