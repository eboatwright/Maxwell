![](/icon/Maxwell_316x316.png)
# Maxwell Chess Engine
 A Chess engine written from scratch in Rust.<br>
 If you use this code verbatim, or as a reference, please credit me!<br><br>
 
 <br><br>
 [Play against Maxwell on Lichess!](https://lichess.org/@/MaxwellOnLC) | [Some of Maxwell's Games](https://www.chess.com/library/collections/maxwells-games-my-chess-engine-2FFU82NM4) | [Maxwell's CCRL](https://computerchess.org.uk/ccrl/404/cgi/engine_details.cgi?print=Details&each_game=1&eng=Maxwell%203.0.8-1%2064-bit#Maxwell_3_0_8-1_64-bit)

## Features - NOT UP TO DATE WITH DEV BRANCH
#### Parameters
 - fen=\<FEN STRING>: Sets up the board by a fen string (Doesn't work for UCI games) (default=STARTING_FEN)
 - debug=\<BOOLEAN>: Toggle debug output that gets outputed per ply (default=true)
 - opening_book=\<BOOLEAN>: Toggle built-in opening book (default=false)
 - time_management=\<BOOLEAN>: Toggle time management, if false the bot will use all the remaining time (default=true)
#### UCI Interface
 - uci, isready, ucinewgame, position, go, stop, and quit commands
 - "position" is only implemented for "position startpos", "position fen" is not yet implemented
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
 - Best move from the previous iteration, otherwise from the transposition table
 - MVV-LVA
 - 2 Killer Moves
 - History Heuristic
 - Penalty for moving a piece to a square an opponent's piece attacks
#### Search
 - Iterative Deepening
 - Alpha-Beta Pruning
 - Late Move Reductions
 - Null Move Pruning
 - Razoring
 - Reverse Futility Pruning
 - Quiescence Search with Delta Pruning
 - Transposition Table
   - UCI Hash option to change max size, default is 256 MB
   - Replacement scheme prefers higher depth and exact evaluation bound
 - Search Extensions
   - Checks
   - Pawn moves to the 2nd or 7th rank
 - Time management
   - If less than 7 moves have been played, it uses 2.5% of it's remaining time, otherwise 7%
   - This value is then also clamped between 0.05 and 20.0 seconds

## Helpful Sources & References
 - [Sebastian Lague's Chess Programming series](https://www.youtube.com/playlist?list=PLFt_AvWsXl0cvHyu32ajwh2qU1i6hl77c)
 - [The Chess Programming Wiki](https://www.chessprogramming.org/Main_Page)
 - [BBC Engine Development](https://www.youtube.com/playlist?list=PLmN0neTso3Jxh8ZIylk74JpwfiWNI76Cs)
 - [Boychesser](https://github.com/analog-hors/Boychesser/)
 - [Lynx](https://github.com/lynx-chess/Lynx/)
 - [Weiawaga](https://github.com/Heiaha/Weiawaga/)
 - [Fruit 2.1](https://github.com/Warpten/Fruit-2.1/)
 - [Perfect 2021 Opening Book](https://sites.google.com/site/computerschess/perfect-2021-books)
 - [Cute Chess](https://cutechess.com/)
 - [PVS Implementation](https://web.archive.org/web/20071030220825/http://www.brucemo.com/compchess/programming/pvs.htm)
 - [LMR Implementation](https://web.archive.org/web/20150212051846/http://www.glaurungchess.com/lmr.html)
 - [Mediocre Chess](https://mediocrechess.blogspot.com/)
 - [Tcheran](https://github.com/jgilchrist/tcheran/)
 - [Rustic (Engine and Book)](https://github.com/mvanthoor/rustic)