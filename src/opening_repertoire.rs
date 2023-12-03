pub const OPENING_REPERTOIRE: [u64; 84] = [
	8171786471666089984, // 1. e4
	9423734941272768512, // 1. e4 e5


	// The Italian
	8930981383581466624, // 1. e4 e5, 2. Nf3
	944878511468838912, // 1. e4 e5, 2. Nf3 Nc6
	122337436142403584, // 1. e4 e5, 2. Nf3 Nc6, 3. Bc4
	197763521490976768, // 1. e4 e5, 2. Nf3 Nc6, 3. Bc4 Bc5
	3930464210546327552, // 1. e4 e5, 2. Nf3 Nc6, 3. Bc4 Bc5, 4. c3 Nf6
	10372430269918478336, // 1. e4 e5, 2. Nf3 Nc6, 3. Bc4 Bc5, 4. c3 Nf6, 5. d3 d6

	5567388809554821120, // 1. e4 e5, 2. Nf3 Nc6, 3. Bc4 Bc5, 4. O-O
	8667471464795996160, // 1. e4 e5, 2. Nf3 Nc6, 3. Bc4 Bc5, 4. O-O Nf6
	11274630412921470976, // 1. e4 e5, 2. Nf3 Nc6, 3. Bc4 Bc5, 4. O-O Nf6, 5. d3


	// The Four Knights
	8676247629169950720, // 1. e4 e5, 2. Nf3 Nc6, 3. Nc3
	5576129789556686848, // 1. e4 e5, 2. Nf3 Nc6, 3. Nc3 Nf6


	// The Scotch?
	9202674211498229760, // 1. e4 e5, 2. Nf3 Nc6, 3. d4
	12277612204043272192, // 1. e4 e5, 2. Nf3 Nc6, 3. d4 exd4


	// The Spanish
	9235287169487077376,  // 1. e4 e5, 2. Nf3 Nc6, 3. Bb5
	1409002710518202368,  // 1. e4 e5, 2. Nf3 Nc6, 3. Bb5 a6
	13142705753754173440, // 1. e4 e5, 2. Nf3 Nc6, 3. Bb5 a6, 4. Ba4
	6083992612698587136,  // 1. e4 e5, 2. Nf3 Nc6, 3. Bb5 a6, 4. Ba4 b5
	7610213931436474368,  // 1. e4 e5, 2. Nf3 Nc6, 3. Bb5 a6, 4. Ba4 b5, 5. Bb3
	6671718359844782080, // 1. e4 e5, 2. Nf3 Nc6, 3. Bb5 a6, 4. Ba4 b5, 5. Bb3 Nf6
	1399347074280980480, // 1. e4 e5, 2. Nf3 Nc6, 3. Bb5 a6, 4. Ba4 b5, 5. Bb3 Nf6, 6. O-O
	1181987506803965952, // 1. e4 e5, 2. Nf3 Nc6, 3. Bb5 a6, 4. Ba4 b5, 5. Bb3 Nf6, 6. O-O Bc5
	17607316011608965120, // 1. e4 e5, 2. Nf3 Nc6, 3. Bb5 a6, 4. Ba4 b5, 5. Bb3 Nf6, 6. O-O Bc5, 7. d3

	11348064320639467520, // 1. e4 e5, 2. Nf3 Nc6, 3. Bb5 a6, 4. Ba4 b5, 5. Bb3 Nf6, 6. O-O Nxe4, 7. Re1
	5194504559279472640, // 1. e4 e5, 2. Nf3 Nc6, 3. Bb5 a6, 4. Ba4 b5, 5. Bb3 Nf6, 6. O-O Nxe4, 7. Re1 d5, 8. d3


	// The Scandinavian
	16891299969288372224, // 1. e4 d5, 2. exd5
	16813911032345919488, // 1. e4 d5, 2. exd5 Qxd5
	11251074063491137536, // 1. e4 d5, 2. exd5 Qxd5, 3. Nc3

	1202857474449735680, // 1. e4 d5, 2. exd5 Qxd5, 3. Nc3 Qa5
	7069523247322103808, // 1. e4 d5, 2. exd5 Qxd5, 3. Nc3 Qa5, 4. d4
	11272309412594712576, // 1. e4 d5, 2. exd5 Qxd5, 3. Nc3 Qa5, 4. d4 c6

	17197212261941248000, // 1. e4 d5, 2. exd5 Qxd5, 3. Nc3 Qd8, 4. d4

	// 1. e4 d5, 2. e5 ?


	// The Vienna


	// The (Open) Sicilian
	// 8286592528036134912, // 1. e4 c5
	7625710173440442368, // 1. e4 c5, 2. Nf3
	2251286616632983552, // 1. e4 c5, 2. Nf3 Nc6
	7896138562985263104, // 1. e4 c5, 2. Nf3 Nc6, 3. d4
	4574087943437680640, // 1. e4 c5, 2. Nf3 Nc6, 3. d4 cxd4
	3768654169527812096, // 1. e4 c5, 2. Nf3 Nc6, 3. d4 cxd4, 4. Nxd4
	91952982064627712, // 1. e4 c5, 2. Nf3 Nc6, 3. d4 cxd4, 4. Nxd4 Nf6
	8372883849851437056, // 1. e4 c5, 2. Nf3 Nc6, 3. d4 cxd4, 4. Nxd4 Nf6, 5. Nc3
	4151396091301986304, // 1. e4 c5, 2. Nf3 Nc6, 3. d4 cxd4, 4. Nxd4 Nf6, 5. Nc3 e6

	3788902500787027968, // 1. e4 c5, 2. Nf3 d6
	5061614629008965632, // 1. e4 c5, 2. Nf3 d6, 3. d4
	1499878171187609600, // 1. e4 c5, 2. Nf3 d6, 3. d4 cxd4
	2298591180272697344, // 1. e4 c5, 2. Nf3 d6, 3. d4 cxd4, 4. Nxd4
	3092953971602489344, // 1. e4 c5, 2. Nf3 d6, 3. d4 cxd4, 4. Nxd4 Nf6
	6888460139225939968, // 1. e4 c5, 2. Nf3 d6, 3. d4 cxd4, 4. Nxd4 Nf6, 5. Nc3
	14717524163589832704, // 1. e4 c5, 2. Nf3 d6, 3. d4 cxd4, 4. Nxd4 Nf6, 5. Nc3 a6



	3935442318160560128, // 1. d4
	560986431812534272, // 1. d4 d5


	// Queen's Gambit
	2776991094699720704, // 1. d4 d5, 2. c4

	// Slav?
	11121701058601025536, // 1. d4 d5, 2. c4 c6

	// Semi-slav?
	4871054970264223744, // 1. d4 d5, 2. c4 e6


	// London
	355674350680014848, // 1. d4 d5, 2. Bf4
	177809103433760768, // 1. d4 d5, 2. Bf4 Nf6, 3. Nc3
	649511686377570304, // 1. d4 d5, 2. Bf4 Nf6, 3. e3


	// Jobava London
	1776365419557289984, // 1. d4 d5, 2. Nc3
	3291206844487303168, // 1. d4 d5, 2. Nc3 Nf6
	4940762633075687424, // 1. d4 d5, 2. Nc3 Nf6, 3. Bf4
	15507724410055294976, // 1. d4 d5, 2. Nc3 Nf6, 3. Bf4 a6
	17280523357489463296, // 1. d4 d5, 2. Nc3 Nf6, 3. Bf4 a6, 4. e3

	251825752314478592, // 1. d4 d5, 2. Nc3 Nf6, 3. Bf4 Bf5
	4294441110966632448, // 1. d4 d5, 2. Nc3 Nf6, 3. Bf4 Bf5, 4. e3

	10205828488506638336, // 1. d4 d5, 2. Nc3 Nf6, 3. Bf4 c5
	11383484606691934208, // 1. d4 d5, 2. Nc3 Nf6, 3. Bf4 c5, 4. e3



	// The English
	1111264857321111552, // 1. c4
	2222858743496835072, // 1. c4 e5
	11603922463129337856, // 1. c4 e5, 2. Nc3
	10665429640316715008, // 1. c4 e5, 2. Nc3 Nf6
	12074508492015140864, // 1. c4 e5, 2. Nc3 Nf6, 3. Nf3
	15095115698021597184, // 1. c4 e5, 2. Nc3 Nf6, 3. Nf3 Nc6

	9834952496219422720, // 1. c4 e5, 2. Nc3 Nf6, 3. Nf3 d6, 4. d4

	17213073610525114368, // 1. c4 c5
	12910594451084148736, // 1. c4 c5, 2. Nc3
	9665279503953297408, // 1. c4 c5, 2. Nc3 Nf6

	14251217708837240832, // 1. c4 c5, 2. Nc3 Nc6

	17711826378535469056, // 1. c4 c5, 2. Nf3
	13891161185845248000, // 1. c4 c5, 2. Nf3 Nf6
	13092961098934517760, // 1. c4 c5, 2. Nf3 Nf6, 3. Nc3
	17878524710812123136, // 1. c4 c5, 2. Nf3 Nf6, 3. Nc3 e6

	9448787313711644672, // 1. c4 c5, 2. Nf3 Nc6
	17749826599903035392, // 1. c4 c5, 2. Nf3 Nc6, 3. Nc3
	13545349935502721024, // 1. c4 c5, 2. Nf3 Nc6, 3. Nc3 e6

	1357902357861498880, // 1. c4 Nf6
	// TODO: continue this line a bit
];