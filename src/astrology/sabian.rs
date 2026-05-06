//! Wave 9.B2 — Sabian Symbols (360 degree-by-degree zodiac meanings).
//!
//! In 1925 astrologer Marc Edmund Jones channeled (with clairvoyant Elsie
//! Wheeler) one symbolic image for every integer degree of the zodiac.
//! 360 symbols total. Each describes a specific quality, scenario, or
//! archetype for a planet found at that exact degree.
//!
//! Convention: the symbol for "Aries 1°" applies to all longitudes from
//! 0°00' Aries up to 0°59' Aries. So a planet at 0.0° Aries OR 0.5° Aries
//! both read as "A WOMAN RISES OUT OF WATER, A SEAL EMBRACES HER" — the
//! first Sabian degree.
//!
//! Use case: narrative depth. Lagrange does not incorporate Sabian (purely
//! symbolic, like horoscope readings). UI surfaces it on planet hover.
//!
//! Reference: Marc Edmund Jones, *The Sabian Symbols in Astrology* (1953),
//! based on the 1925 channeling. The symbol images are part of the public
//! domain (1920s American occult publication, no extant copyright).

/// One Sabian symbol — the image-phrase plus a brief keynote summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SabianSymbol {
    /// Sign (Aries, Taurus, …, Pisces).
    pub sign: &'static str,
    /// Degree within the sign — 1..=30.
    pub degree: u8,
    /// The full symbolic image (1-2 short sentences).
    pub image: &'static str,
    /// One-line interpretive keynote.
    pub keynote: &'static str,
}

/// Wave 9.B2 — Look up the Sabian symbol for any ecliptic longitude.
///
/// Convention: floor(degree-within-sign) + 1 → 1..=30.
/// So 0.0° Aries → degree 1, 0.99° Aries → degree 1, 1.0° Aries → degree 2.
pub fn sabian_for_longitude(lon: f64) -> SabianSymbol {
    let mut l = lon % 360.0;
    if l < 0.0 { l += 360.0; }
    // sign_index 0..11, degree_in_sign 0.0..30.0
    let sign_index = (l / 30.0).floor() as usize % 12;
    let degree_in_sign = l - sign_index as f64 * 30.0;
    let degree = (degree_in_sign.floor() as u8 + 1).min(30);
    let global_index = sign_index * 30 + (degree as usize - 1);
    SABIAN_SYMBOLS[global_index]
}

/// Helper for UI rendering: "Aries 1° — A woman rises out of water."
pub fn sabian_label(s: SabianSymbol) -> String {
    format!("{} {}° — {}", s.sign, s.degree, s.image)
}

/// All 360 Sabian symbols, ordered Aries 1 → Pisces 30.
const SABIAN_SYMBOLS: [SabianSymbol; 360] = [
    // ── ARIES ─────────────────────────────────────────────────────
    SabianSymbol { sign: "Aries", degree: 1,  image: "A woman rises out of water, a seal embraces her.", keynote: "Birth of new awareness." },
    SabianSymbol { sign: "Aries", degree: 2,  image: "A comedian entertains a group.", keynote: "Joy of self-expression." },
    SabianSymbol { sign: "Aries", degree: 3,  image: "A cameo profile of a man.", keynote: "Rooted identity." },
    SabianSymbol { sign: "Aries", degree: 4,  image: "Two lovers strolling on a secluded walk.", keynote: "Inner harmony of duality." },
    SabianSymbol { sign: "Aries", degree: 5,  image: "A triangle with wings.", keynote: "Idealism in motion." },
    SabianSymbol { sign: "Aries", degree: 6,  image: "A square brightly lighted on one side.", keynote: "Clarity emerging from form." },
    SabianSymbol { sign: "Aries", degree: 7,  image: "A man successfully expressing himself in two realms.", keynote: "Bridge across dimensions." },
    SabianSymbol { sign: "Aries", degree: 8,  image: "A large hat with streamers flying east.", keynote: "Mind moving with the wind." },
    SabianSymbol { sign: "Aries", degree: 9,  image: "A crystal gazer.", keynote: "Inner vision focused outward." },
    SabianSymbol { sign: "Aries", degree: 10, image: "A teacher gives new symbolic forms to traditional images.", keynote: "Renewing tradition." },
    SabianSymbol { sign: "Aries", degree: 11, image: "The ruler of a nation.", keynote: "Authority embodied." },
    SabianSymbol { sign: "Aries", degree: 12, image: "A flock of wild geese.", keynote: "Group instinct, common purpose." },
    SabianSymbol { sign: "Aries", degree: 13, image: "An unsuccessful bomb explosion.", keynote: "Frustrated outburst." },
    SabianSymbol { sign: "Aries", degree: 14, image: "A serpent coiling near a man and a woman.", keynote: "Awakening through tension." },
    SabianSymbol { sign: "Aries", degree: 15, image: "An Indian weaving a blanket.", keynote: "Patient creative integration." },
    SabianSymbol { sign: "Aries", degree: 16, image: "Brightly clad nature spirits dance in twilight.", keynote: "Joy at the edge of mystery." },
    SabianSymbol { sign: "Aries", degree: 17, image: "Two prim spinsters sitting together in silence.", keynote: "Reserved companionship." },
    SabianSymbol { sign: "Aries", degree: 18, image: "An empty hammock.", keynote: "Rest and replenishment." },
    SabianSymbol { sign: "Aries", degree: 19, image: "The magic carpet of oriental imagery.", keynote: "Imagination as transport." },
    SabianSymbol { sign: "Aries", degree: 20, image: "A young girl feeding birds in winter.", keynote: "Generosity in scarcity." },
    SabianSymbol { sign: "Aries", degree: 21, image: "A pugilist enters the ring.", keynote: "Test of personal force." },
    SabianSymbol { sign: "Aries", degree: 22, image: "The gate to the garden of all desires.", keynote: "Threshold of fulfillment." },
    SabianSymbol { sign: "Aries", degree: 23, image: "A pregnant woman in a sunlit field.", keynote: "Fertile becoming." },
    SabianSymbol { sign: "Aries", degree: 24, image: "An open window and a net curtain blowing into a cornucopia.", keynote: "Welcoming abundance." },
    SabianSymbol { sign: "Aries", degree: 25, image: "A double promise reveals its inner and outer meaning.", keynote: "Layered commitment." },
    SabianSymbol { sign: "Aries", degree: 26, image: "A man possessed of more gifts than he can hold.", keynote: "Overflowing potential." },
    SabianSymbol { sign: "Aries", degree: 27, image: "Lost opportunity regained in imagination.", keynote: "Redemption through reflection." },
    SabianSymbol { sign: "Aries", degree: 28, image: "A large disappointed audience.", keynote: "Reckoning with expectation." },
    SabianSymbol { sign: "Aries", degree: 29, image: "A celestial choir singing.", keynote: "Harmonic resonance." },
    SabianSymbol { sign: "Aries", degree: 30, image: "A duck pond and its brood.", keynote: "Quiet domestic continuity." },

    // ── TAURUS ────────────────────────────────────────────────────
    SabianSymbol { sign: "Taurus", degree: 1,  image: "A clear mountain stream.", keynote: "Pure, life-giving inflow." },
    SabianSymbol { sign: "Taurus", degree: 2,  image: "An electrical storm.", keynote: "Power released, sky cleansed." },
    SabianSymbol { sign: "Taurus", degree: 3,  image: "Steps up to a lawn blooming with clover.", keynote: "Gentle ascent into prosperity." },
    SabianSymbol { sign: "Taurus", degree: 4,  image: "The pot of gold at the end of the rainbow.", keynote: "Promise made tangible." },
    SabianSymbol { sign: "Taurus", degree: 5,  image: "A widow at an open grave.", keynote: "Letting go that life may continue." },
    SabianSymbol { sign: "Taurus", degree: 6,  image: "A bridge being built across a gorge.", keynote: "Spanning what divides." },
    SabianSymbol { sign: "Taurus", degree: 7,  image: "A woman of Samaria comes to draw water from the well.", keynote: "Daily nourishment as encounter." },
    SabianSymbol { sign: "Taurus", degree: 8,  image: "A sleigh without snow.", keynote: "Form awaiting its element." },
    SabianSymbol { sign: "Taurus", degree: 9,  image: "A Christmas tree decorated.", keynote: "Festive shared meaning." },
    SabianSymbol { sign: "Taurus", degree: 10, image: "A Red Cross nurse.", keynote: "Service in crisis." },
    SabianSymbol { sign: "Taurus", degree: 11, image: "A woman watering flowers in her garden.", keynote: "Patient tending of beauty." },
    SabianSymbol { sign: "Taurus", degree: 12, image: "A young couple walk down Main Street, window-shopping.", keynote: "Anticipation of choice." },
    SabianSymbol { sign: "Taurus", degree: 13, image: "A man handling baggage.", keynote: "Discipline of necessities." },
    SabianSymbol { sign: "Taurus", degree: 14, image: "On the beach, children play while shellfish grope at the edge of the water.", keynote: "Innocence at the edge of the unknown." },
    SabianSymbol { sign: "Taurus", degree: 15, image: "A man with rakish silk hat braving the storm.", keynote: "Defiant elegance." },
    SabianSymbol { sign: "Taurus", degree: 16, image: "An old teacher fails to interest her pupils in traditional knowledge.", keynote: "Old forms losing potency." },
    SabianSymbol { sign: "Taurus", degree: 17, image: "A symbolical battle between swords and torches.", keynote: "Conflict of force vs illumination." },
    SabianSymbol { sign: "Taurus", degree: 18, image: "A woman holds a bag out of a window.", keynote: "Releasing what is no longer needed." },
    SabianSymbol { sign: "Taurus", degree: 19, image: "A new continent rising out of the ocean.", keynote: "Surfacing of latent worlds." },
    SabianSymbol { sign: "Taurus", degree: 20, image: "Wisps of clouds streaming across the sky.", keynote: "Ephemeral patterns of mind." },
    SabianSymbol { sign: "Taurus", degree: 21, image: "A finger pointing in an open book.", keynote: "Direct revelation through study." },
    SabianSymbol { sign: "Taurus", degree: 22, image: "White dove flying over troubled waters.", keynote: "Peace amid agitation." },
    SabianSymbol { sign: "Taurus", degree: 23, image: "A jewelry shop filled with valuable gems.", keynote: "Treasures of refined experience." },
    SabianSymbol { sign: "Taurus", degree: 24, image: "An Indian warrior riding fiercely.", keynote: "Concentrated will in motion." },
    SabianSymbol { sign: "Taurus", degree: 25, image: "A large well-kept public park.", keynote: "Civic shared abundance." },
    SabianSymbol { sign: "Taurus", degree: 26, image: "A Spanish gallant serenades his beloved.", keynote: "Devotion expressed as art." },
    SabianSymbol { sign: "Taurus", degree: 27, image: "An old Indian woman selling beads.", keynote: "Wisdom packaged for exchange." },
    SabianSymbol { sign: "Taurus", degree: 28, image: "A mature woman, keeping up with the times, has her hair bobbed.", keynote: "Adapting form to era." },
    SabianSymbol { sign: "Taurus", degree: 29, image: "Two cobblers working at a table.", keynote: "Quiet collaboration on essentials." },
    SabianSymbol { sign: "Taurus", degree: 30, image: "A peacock parading on an ancient lawn.", keynote: "Display and inheritance." },

    // ── GEMINI ────────────────────────────────────────────────────
    SabianSymbol { sign: "Gemini", degree: 1,  image: "A glass-bottomed boat reveals undersea wonders.", keynote: "Discovery through unusual lenses." },
    SabianSymbol { sign: "Gemini", degree: 2,  image: "Santa Claus filling stockings furtively.", keynote: "Joy of unseen giving." },
    SabianSymbol { sign: "Gemini", degree: 3,  image: "The garden of the Tuileries in Paris.", keynote: "Cultivated beauty as legacy." },
    SabianSymbol { sign: "Gemini", degree: 4,  image: "Holly and mistletoe bring Christmas spirit.", keynote: "Seasonal ritual unites." },
    SabianSymbol { sign: "Gemini", degree: 5,  image: "A revolutionary magazine asking for action.", keynote: "Catalyst of social change." },
    SabianSymbol { sign: "Gemini", degree: 6,  image: "Workmen drilling for oil.", keynote: "Boring through to vital resource." },
    SabianSymbol { sign: "Gemini", degree: 7,  image: "An old-fashioned well.", keynote: "Steady classical source." },
    SabianSymbol { sign: "Gemini", degree: 8,  image: "Aroused strikers surround a factory.", keynote: "Collective demand for fairness." },
    SabianSymbol { sign: "Gemini", degree: 9,  image: "A quiver filled with arrows.", keynote: "Readiness with multiple options." },
    SabianSymbol { sign: "Gemini", degree: 10, image: "An airplane performing a nose dive.", keynote: "Dramatic plunge into experience." },
    SabianSymbol { sign: "Gemini", degree: 11, image: "Newly opened lands offer the pioneer new opportunities.", keynote: "Frontier as opening." },
    SabianSymbol { sign: "Gemini", degree: 12, image: "A topsy-turvy circus performance.", keynote: "Inversion as entertainment." },
    SabianSymbol { sign: "Gemini", degree: 13, image: "A famous pianist giving a concert.", keynote: "Mastery shared with the public." },
    SabianSymbol { sign: "Gemini", degree: 14, image: "A conversation by telepathy.", keynote: "Communication beyond the visible." },
    SabianSymbol { sign: "Gemini", degree: 15, image: "Two Dutch children talking.", keynote: "Innocent peer dialogue." },
    SabianSymbol { sign: "Gemini", degree: 16, image: "A woman activist in an emotional speech.", keynote: "Voice carrying conviction." },
    SabianSymbol { sign: "Gemini", degree: 17, image: "The head of a robust youth changes into that of a mature thinker.", keynote: "Maturation of perception." },
    SabianSymbol { sign: "Gemini", degree: 18, image: "Two Chinese men talking Chinese in a Western crowd.", keynote: "Inner-circle communication." },
    SabianSymbol { sign: "Gemini", degree: 19, image: "A large archaic volume reveals a traditional wisdom.", keynote: "Old books, new relevance." },
    SabianSymbol { sign: "Gemini", degree: 20, image: "A modern cafeteria displays an abundance of food.", keynote: "Choice as social ritual." },
    SabianSymbol { sign: "Gemini", degree: 21, image: "A tumultuous labor demonstration.", keynote: "Public pressure for change." },
    SabianSymbol { sign: "Gemini", degree: 22, image: "A barn dance.", keynote: "Communal celebration of harvest." },
    SabianSymbol { sign: "Gemini", degree: 23, image: "Three fledglings in a nest high in a tree.", keynote: "Sheltered potential." },
    SabianSymbol { sign: "Gemini", degree: 24, image: "Children skating on ice.", keynote: "Skill on slippery ground." },
    SabianSymbol { sign: "Gemini", degree: 25, image: "A gardener trimming large palm trees.", keynote: "Shaping nature's exuberance." },
    SabianSymbol { sign: "Gemini", degree: 26, image: "Winter frost in the woods.", keynote: "Crystallized stillness." },
    SabianSymbol { sign: "Gemini", degree: 27, image: "A young gypsy emerging from the woods gazes at far cities.", keynote: "Wanderer's first glimpse of civilization." },
    SabianSymbol { sign: "Gemini", degree: 28, image: "A man declared bankrupt.", keynote: "Collapse forcing reassessment." },
    SabianSymbol { sign: "Gemini", degree: 29, image: "The first mockingbird of spring.", keynote: "Awakening of imitation as art." },
    SabianSymbol { sign: "Gemini", degree: 30, image: "A parade of bathing beauties before large beach crowds.", keynote: "Public display of physical ideal." },

    // ── CANCER ────────────────────────────────────────────────────
    SabianSymbol { sign: "Cancer", degree: 1,  image: "On a ship, sailors lower an old flag and raise a new one.", keynote: "Ritual of renewed allegiance." },
    SabianSymbol { sign: "Cancer", degree: 2,  image: "A man on a magic carpet observes vast vistas below him.", keynote: "Detached observation of life." },
    SabianSymbol { sign: "Cancer", degree: 3,  image: "A man bundled in fur leads a shaggy deer.", keynote: "Mastery over wild instinct." },
    SabianSymbol { sign: "Cancer", degree: 4,  image: "A cat arguing with a mouse.", keynote: "Hierarchy within tension." },
    SabianSymbol { sign: "Cancer", degree: 5,  image: "At a railroad crossing, an automobile is wrecked by a train.", keynote: "Collision of fast vs steady." },
    SabianSymbol { sign: "Cancer", degree: 6,  image: "Game birds feathering their nests.", keynote: "Instinctual care for offspring." },
    SabianSymbol { sign: "Cancer", degree: 7,  image: "Two fairies dancing on a moonlit night.", keynote: "Pure imaginative play." },
    SabianSymbol { sign: "Cancer", degree: 8,  image: "A group of rabbits dressed in human clothes.", keynote: "Innocence in performance." },
    SabianSymbol { sign: "Cancer", degree: 9,  image: "A tiny nude miss reaches into the water for a fish.", keynote: "Innocent pursuit of nourishment." },
    SabianSymbol { sign: "Cancer", degree: 10, image: "A large diamond not completely cut.", keynote: "Potential awaiting refinement." },
    SabianSymbol { sign: "Cancer", degree: 11, image: "A clown caricaturing well-known personalities.", keynote: "Truth through exaggeration." },
    SabianSymbol { sign: "Cancer", degree: 12, image: "A Chinese woman nursing a baby whose aura reveals him to be a reincarnation of a great teacher.", keynote: "Ancient wisdom in fresh form." },
    SabianSymbol { sign: "Cancer", degree: 13, image: "One hand slightly flexed with a very prominent thumb.", keynote: "Will revealed in detail." },
    SabianSymbol { sign: "Cancer", degree: 14, image: "A very old man facing a vast dark space to the northeast.", keynote: "Experience facing the unknown." },
    SabianSymbol { sign: "Cancer", degree: 15, image: "A group of people who have overeaten and enjoyed it.", keynote: "Unrestrained satisfaction." },
    SabianSymbol { sign: "Cancer", degree: 16, image: "A man studying a mandala in front of him with the help of a very ancient book.", keynote: "Tradition decoded for personal use." },
    SabianSymbol { sign: "Cancer", degree: 17, image: "The germ grows into knowledge and life.", keynote: "Seed becomes structure." },
    SabianSymbol { sign: "Cancer", degree: 18, image: "A hen scratching for her chicks.", keynote: "Maternal provision." },
    SabianSymbol { sign: "Cancer", degree: 19, image: "A priest performing a marriage ceremony.", keynote: "Sanctified union." },
    SabianSymbol { sign: "Cancer", degree: 20, image: "Venetian gondoliers giving a serenade.", keynote: "Beauty that moves with the current." },
    SabianSymbol { sign: "Cancer", degree: 21, image: "A famous singer is proving her virtuosity during an operatic performance.", keynote: "Mastery on the public stage." },
    SabianSymbol { sign: "Cancer", degree: 22, image: "A young woman awaiting a sailboat.", keynote: "Patient anticipation." },
    SabianSymbol { sign: "Cancer", degree: 23, image: "The meeting of a literary society.", keynote: "Communal cultivation of mind." },
    SabianSymbol { sign: "Cancer", degree: 24, image: "A woman and two men castaways on a small island.", keynote: "Forced intimacy in isolation." },
    SabianSymbol { sign: "Cancer", degree: 25, image: "A will-full man is overshadowed by a descent of superior power.", keynote: "Surrender to the greater." },
    SabianSymbol { sign: "Cancer", degree: 26, image: "Guests are reading in the library of a luxurious home.", keynote: "Quiet privilege of access." },
    SabianSymbol { sign: "Cancer", degree: 27, image: "A storm in a canyon.", keynote: "Cathartic concentration of force." },
    SabianSymbol { sign: "Cancer", degree: 28, image: "An Indian girl introduces her white lover to her assembled tribe.", keynote: "Bridging worlds." },
    SabianSymbol { sign: "Cancer", degree: 29, image: "A Greek muse weighing newborn twins in golden scales.", keynote: "Balanced beginnings." },
    SabianSymbol { sign: "Cancer", degree: 30, image: "A daughter of the American Revolution.", keynote: "Legacy guarded with pride." },

    // ── LEO ───────────────────────────────────────────────────────
    SabianSymbol { sign: "Leo", degree: 1,  image: "Blood rushes to a man's head as his vital energies are mobilized.", keynote: "Awakening of full force." },
    SabianSymbol { sign: "Leo", degree: 2,  image: "An epidemic of mumps.", keynote: "Shared affliction binding community." },
    SabianSymbol { sign: "Leo", degree: 3,  image: "A mature woman, her hair adorned with a white dove.", keynote: "Wisdom married to spirit." },
    SabianSymbol { sign: "Leo", degree: 4,  image: "A man formally dressed stands near trophies he brought back from a hunting expedition.", keynote: "Pride of conquest displayed." },
    SabianSymbol { sign: "Leo", degree: 5,  image: "Rock formations tower over a deep canyon.", keynote: "Ancient form witnesses depth." },
    SabianSymbol { sign: "Leo", degree: 6,  image: "A conservative, old-fashioned lady is confronted by a hippie girl.", keynote: "Generations meeting." },
    SabianSymbol { sign: "Leo", degree: 7,  image: "The constellations of stars in the sky.", keynote: "Pattern revealing cosmic order." },
    SabianSymbol { sign: "Leo", degree: 8,  image: "A communist activist spreading his revolutionary ideals.", keynote: "Conviction seeking converts." },
    SabianSymbol { sign: "Leo", degree: 9,  image: "Glass blowers shape beautiful vases with their controlled breathing.", keynote: "Form shaped by life-breath." },
    SabianSymbol { sign: "Leo", degree: 10, image: "Early morning dew sparkles as sunlight floods the field.", keynote: "Freshness illuminated." },
    SabianSymbol { sign: "Leo", degree: 11, image: "Children play on a swing hanging from the branches of a huge oak.", keynote: "Joy supported by lineage." },
    SabianSymbol { sign: "Leo", degree: 12, image: "An evening party of adults on a lawn illumined by fancy lanterns.", keynote: "Civilized social warmth." },
    SabianSymbol { sign: "Leo", degree: 13, image: "An old sea captain rocking on the porch of his cottage.", keynote: "Rest after a life at sea." },
    SabianSymbol { sign: "Leo", degree: 14, image: "The constellations of stars shine brightly in the night sky.", keynote: "Vivid cosmic guidance." },
    SabianSymbol { sign: "Leo", degree: 15, image: "A pageant, with its spectacular floats, moves along a street crowded with cheering people.", keynote: "Celebration of group pride." },
    SabianSymbol { sign: "Leo", degree: 16, image: "The storm ended, all nature rejoices in brilliant sunshine.", keynote: "Catharsis followed by clarity." },
    SabianSymbol { sign: "Leo", degree: 17, image: "A volunteer church choir makes a social event of a rehearsal.", keynote: "Devotion as community." },
    SabianSymbol { sign: "Leo", degree: 18, image: "A chemist conducts an experiment for his students.", keynote: "Knowledge demonstrated in action." },
    SabianSymbol { sign: "Leo", degree: 19, image: "A houseboat party.", keynote: "Pleasure on shifting waters." },
    SabianSymbol { sign: "Leo", degree: 20, image: "American Indians perform a ritual to the sun.", keynote: "Honoring the source of vitality." },
    SabianSymbol { sign: "Leo", degree: 21, image: "Intoxicated chickens dizzily flap their wings, trying to fly.", keynote: "Misdirected exuberance." },
    SabianSymbol { sign: "Leo", degree: 22, image: "A carrier pigeon fulfilling its mission.", keynote: "Faithful service over distance." },
    SabianSymbol { sign: "Leo", degree: 23, image: "A bareback rider in a circus thrills excited crowds.", keynote: "Skill displayed at the edge." },
    SabianSymbol { sign: "Leo", degree: 24, image: "Totally concentrated upon inner spiritual attainment, a man is sitting in a state of complete neglect of his body.", keynote: "Inner pursuit at outer cost." },
    SabianSymbol { sign: "Leo", degree: 25, image: "A large camel crossing a vast and forbidding desert.", keynote: "Endurance through emptiness." },
    SabianSymbol { sign: "Leo", degree: 26, image: "After a heavy storm, a rainbow.", keynote: "Promise after difficulty." },
    SabianSymbol { sign: "Leo", degree: 27, image: "Daybreak — the luminescence of dawn in the eastern sky.", keynote: "New cycle promised." },
    SabianSymbol { sign: "Leo", degree: 28, image: "Many little birds on the limb of a large tree.", keynote: "Many voices in shared structure." },
    SabianSymbol { sign: "Leo", degree: 29, image: "A mermaid emerges from the ocean waves ready for rebirth in human form.", keynote: "Soul stepping into manifestation." },
    SabianSymbol { sign: "Leo", degree: 30, image: "An unsealed letter.", keynote: "Truth ready to be received." },

    // ── VIRGO ─────────────────────────────────────────────────────
    SabianSymbol { sign: "Virgo", degree: 1,  image: "In a portrait, the significant features of a man's head are artistically emphasized.", keynote: "Essence captured in art." },
    SabianSymbol { sign: "Virgo", degree: 2,  image: "A large white cross dominating the landscape stands alone on top of a high hill.", keynote: "Pure principle as landmark." },
    SabianSymbol { sign: "Virgo", degree: 3,  image: "Two angels bringing protection.", keynote: "Spiritual guardianship." },
    SabianSymbol { sign: "Virgo", degree: 4,  image: "Black and white children play happily together.", keynote: "Innocent unity transcending boundaries." },
    SabianSymbol { sign: "Virgo", degree: 5,  image: "A man becoming aware of nature spirits and normally unseen spiritual energies.", keynote: "Subtle realms perceived." },
    SabianSymbol { sign: "Virgo", degree: 6,  image: "A merry-go-round.", keynote: "Cyclical pleasure." },
    SabianSymbol { sign: "Virgo", degree: 7,  image: "A harem.", keynote: "Multiplicity in personal relationships." },
    SabianSymbol { sign: "Virgo", degree: 8,  image: "First dancing instruction.", keynote: "Beginning of patterned movement." },
    SabianSymbol { sign: "Virgo", degree: 9,  image: "An expressionist painter making a futuristic drawing.", keynote: "Vision projecting forward." },
    SabianSymbol { sign: "Virgo", degree: 10, image: "Two heads looking out and beyond the shadows.", keynote: "Bifocal awareness piercing illusion." },
    SabianSymbol { sign: "Virgo", degree: 11, image: "A boy molded in his mother's aspirations for him.", keynote: "Identity shaped by another's vision." },
    SabianSymbol { sign: "Virgo", degree: 12, image: "A bride with her veil snatched away.", keynote: "Sudden revelation of truth." },
    SabianSymbol { sign: "Virgo", degree: 13, image: "A strong hand supplanting political hysteria.", keynote: "Steady authority restoring order." },
    SabianSymbol { sign: "Virgo", degree: 14, image: "A family tree.", keynote: "Lineage made visible." },
    SabianSymbol { sign: "Virgo", degree: 15, image: "A fine lace ornamental handkerchief.", keynote: "Refined symbolic expression." },
    SabianSymbol { sign: "Virgo", degree: 16, image: "An orangutan in a zoological garden.", keynote: "Wild self in confinement." },
    SabianSymbol { sign: "Virgo", degree: 17, image: "A volcanic eruption.", keynote: "Suppressed energy bursting forth." },
    SabianSymbol { sign: "Virgo", degree: 18, image: "Two girls playing with a ouija board.", keynote: "Inquiry into unseen levels." },
    SabianSymbol { sign: "Virgo", degree: 19, image: "A swimming race.", keynote: "Test of stamina in a fluid medium." },
    SabianSymbol { sign: "Virgo", degree: 20, image: "An automobile caravan.", keynote: "Group movement toward shared goal." },
    SabianSymbol { sign: "Virgo", degree: 21, image: "A girls' basketball team.", keynote: "Coordinated youthful ambition." },
    SabianSymbol { sign: "Virgo", degree: 22, image: "A royal coat of arms.", keynote: "Ancestral identity emblazoned." },
    SabianSymbol { sign: "Virgo", degree: 23, image: "A lion-tamer rushes fearlessly into the circus arena.", keynote: "Courage facing primal force." },
    SabianSymbol { sign: "Virgo", degree: 24, image: "Mary and her white lamb.", keynote: "Innocence walking beside us." },
    SabianSymbol { sign: "Virgo", degree: 25, image: "A flag at half-mast in front of a public building.", keynote: "Public mourning, communal loss." },
    SabianSymbol { sign: "Virgo", degree: 26, image: "A boy with a censer serves the priest near the altar.", keynote: "Apprenticeship to the sacred." },
    SabianSymbol { sign: "Virgo", degree: 27, image: "Aristocratic elderly ladies drinking afternoon tea in a luxurious garden.", keynote: "Cultivated leisure of established class." },
    SabianSymbol { sign: "Virgo", degree: 28, image: "A baldheaded man who has seized power.", keynote: "Authority claimed without inheritance." },
    SabianSymbol { sign: "Virgo", degree: 29, image: "A man gaining secret knowledge from an ancient scroll he is reading.", keynote: "Hidden wisdom decoded." },
    SabianSymbol { sign: "Virgo", degree: 30, image: "Having an urgent task to complete, a man doesn't look to any distractions.", keynote: "Single-minded discipline." },

    // ── LIBRA ─────────────────────────────────────────────────────
    SabianSymbol { sign: "Libra", degree: 1,  image: "A butterfly preserved and made perfect with a dart through it.", keynote: "Fixing beauty by arresting it." },
    SabianSymbol { sign: "Libra", degree: 2,  image: "The light of the sixth race transmuted into the seventh.", keynote: "Evolution at the threshold." },
    SabianSymbol { sign: "Libra", degree: 3,  image: "The dawn of a new day reveals everything changed.", keynote: "Fresh perception transforming reality." },
    SabianSymbol { sign: "Libra", degree: 4,  image: "A group of young people sit in spiritual communion around a campfire.", keynote: "Shared seeking under stars." },
    SabianSymbol { sign: "Libra", degree: 5,  image: "A man teaching the true inner knowledge.", keynote: "Mentor transmitting essence." },
    SabianSymbol { sign: "Libra", degree: 6,  image: "The ideals of a man abundantly crystallized.", keynote: "Inner vision made permanent." },
    SabianSymbol { sign: "Libra", degree: 7,  image: "A woman feeding chickens and protecting them from the hawks.", keynote: "Nurture combined with vigilance." },
    SabianSymbol { sign: "Libra", degree: 8,  image: "A blazing fireplace in a deserted home.", keynote: "Continuity sustained in absence." },
    SabianSymbol { sign: "Libra", degree: 9,  image: "Three old masters hanging in a special room in an art gallery.", keynote: "Curated cultural touchstones." },
    SabianSymbol { sign: "Libra", degree: 10, image: "A canoe approaching safety through dangerous waters.", keynote: "Skillful navigation toward shore." },
    SabianSymbol { sign: "Libra", degree: 11, image: "A professor peering over his glasses at his students.", keynote: "Authority assessing potential." },
    SabianSymbol { sign: "Libra", degree: 12, image: "Miners are emerging from a deep coal mine.", keynote: "Returning from labor in darkness." },
    SabianSymbol { sign: "Libra", degree: 13, image: "Children blowing soap bubbles.", keynote: "Innocent delight in transient beauty." },
    SabianSymbol { sign: "Libra", degree: 14, image: "In the heat of the noon hour, a man takes a siesta.", keynote: "Restorative pause within action." },
    SabianSymbol { sign: "Libra", degree: 15, image: "Circular paths.", keynote: "Cycles inviting return." },
    SabianSymbol { sign: "Libra", degree: 16, image: "After a stormy voyage, the boat is being safely moored.", keynote: "Safe arrival after struggle." },
    SabianSymbol { sign: "Libra", degree: 17, image: "A retired sea captain watches ships entering and leaving the harbor.", keynote: "Detached observation of cycles." },
    SabianSymbol { sign: "Libra", degree: 18, image: "Two men placed under arrest.", keynote: "Public consequence of deeds." },
    SabianSymbol { sign: "Libra", degree: 19, image: "A gang of robbers in hiding.", keynote: "Concealed antisocial intent." },
    SabianSymbol { sign: "Libra", degree: 20, image: "A Jewish rabbi performing his duties.", keynote: "Tradition embodied in practice." },
    SabianSymbol { sign: "Libra", degree: 21, image: "A crowd upon a beach.", keynote: "Mass gathering at the boundary." },
    SabianSymbol { sign: "Libra", degree: 22, image: "A child giving birds a drink at a fountain.", keynote: "Innocent generosity." },
    SabianSymbol { sign: "Libra", degree: 23, image: "Chanticleer's voice heralds the rising sun with exuberant tones.", keynote: "Bold proclamation of new day." },
    SabianSymbol { sign: "Libra", degree: 24, image: "A third wing on the left side of a butterfly.", keynote: "Mutation toward greater complexity." },
    SabianSymbol { sign: "Libra", degree: 25, image: "The sight of an autumn leaf brings to a pilgrim the sudden revelation of the mystery of life and death.", keynote: "Sudden insight into impermanence." },
    SabianSymbol { sign: "Libra", degree: 26, image: "An eagle and a large white dove turning into each other.", keynote: "Power and peace inter-transforming." },
    SabianSymbol { sign: "Libra", degree: 27, image: "An airplane sails high in the clear sky.", keynote: "Mind operating at altitude." },
    SabianSymbol { sign: "Libra", degree: 28, image: "A man alone in deep gloom: unnoticed, angels are coming to his help.", keynote: "Help arriving unrecognized." },
    SabianSymbol { sign: "Libra", degree: 29, image: "Mankind's vast and enduring effort to reach for knowledge transferable from generation to generation.", keynote: "Cumulative civilizational wisdom." },
    SabianSymbol { sign: "Libra", degree: 30, image: "Three mounds of knowledge on a philosopher's head.", keynote: "Stratified mastery of mind." },

    // ── SCORPIO ───────────────────────────────────────────────────
    SabianSymbol { sign: "Scorpio", degree: 1,  image: "A sightseeing bus filled with tourists.", keynote: "Mass observation of culture." },
    SabianSymbol { sign: "Scorpio", degree: 2,  image: "A delicate, broken bottle, fragrant with perfume.", keynote: "Beauty released through breakage." },
    SabianSymbol { sign: "Scorpio", degree: 3,  image: "A house-raising party in a small village enlists the neighbors' cooperation.", keynote: "Community building together." },
    SabianSymbol { sign: "Scorpio", degree: 4,  image: "A youth holding a lighted candle in a devotional ritual gains a sense of the great responsibility.", keynote: "Initiation into sacred care." },
    SabianSymbol { sign: "Scorpio", degree: 5,  image: "A massive, rocky shore resists the pounding of the sea.", keynote: "Endurance against persistent force." },
    SabianSymbol { sign: "Scorpio", degree: 6,  image: "A gold rush tears men away from their native soil.", keynote: "Greed as displacement." },
    SabianSymbol { sign: "Scorpio", degree: 7,  image: "Deep-sea divers.", keynote: "Plunging into depth for hidden treasure." },
    SabianSymbol { sign: "Scorpio", degree: 8,  image: "The moon shining across a lake.", keynote: "Reflection illuminating the unconscious." },
    SabianSymbol { sign: "Scorpio", degree: 9,  image: "A dentist at work.", keynote: "Skilled extraction of decay." },
    SabianSymbol { sign: "Scorpio", degree: 10, image: "A fellowship supper reunites old comrades.", keynote: "Bonds renewed across time." },
    SabianSymbol { sign: "Scorpio", degree: 11, image: "A drowning man being rescued.", keynote: "Saved from being overwhelmed." },
    SabianSymbol { sign: "Scorpio", degree: 12, image: "An official embassy ball.", keynote: "Diplomacy as ceremony." },
    SabianSymbol { sign: "Scorpio", degree: 13, image: "An inventor performs a laboratory experiment.", keynote: "Solitary breakthrough." },
    SabianSymbol { sign: "Scorpio", degree: 14, image: "Telephone linemen at work installing new connections.", keynote: "Building infrastructure of communication." },
    SabianSymbol { sign: "Scorpio", degree: 15, image: "Children playing around five mounds of sand.", keynote: "Construction games hinting at adult work." },
    SabianSymbol { sign: "Scorpio", degree: 16, image: "A girl's face breaking into a smile.", keynote: "Sudden warmth dissolving distance." },
    SabianSymbol { sign: "Scorpio", degree: 17, image: "A woman, fecundated by her own spirit, is great with child.", keynote: "Inner generation made visible." },
    SabianSymbol { sign: "Scorpio", degree: 18, image: "A path through woods rich in autumn coloring.", keynote: "Beauty as guide through maturity." },
    SabianSymbol { sign: "Scorpio", degree: 19, image: "A parrot listening, then talking, repeats a conversation he has overheard.", keynote: "Imitation without comprehension." },
    SabianSymbol { sign: "Scorpio", degree: 20, image: "A woman drawing aside two dark curtains that closed the entrance to a sacred pathway.", keynote: "Threshold of mystery breached." },
    SabianSymbol { sign: "Scorpio", degree: 21, image: "Obeying his conscience, a soldier resists orders.", keynote: "Higher law overriding command." },
    SabianSymbol { sign: "Scorpio", degree: 22, image: "Hunters shooting wild ducks.", keynote: "Pursuit of fleeting opportunity." },
    SabianSymbol { sign: "Scorpio", degree: 23, image: "A rabbit metamorphoses into a nature spirit.", keynote: "Animal becoming archetype." },
    SabianSymbol { sign: "Scorpio", degree: 24, image: "Crowds coming down the mountain to listen to one man.", keynote: "Concentration of seekers around a teacher." },
    SabianSymbol { sign: "Scorpio", degree: 25, image: "An X-ray photograph.", keynote: "Penetrating sight." },
    SabianSymbol { sign: "Scorpio", degree: 26, image: "American Indians making camp after moving into a new territory.", keynote: "Establishing roots in new ground." },
    SabianSymbol { sign: "Scorpio", degree: 27, image: "A military band marches noisily on through the city streets.", keynote: "Force on parade." },
    SabianSymbol { sign: "Scorpio", degree: 28, image: "The king of the fairies approaching his domain.", keynote: "Inner sovereign returning." },
    SabianSymbol { sign: "Scorpio", degree: 29, image: "An Indian woman pleading to the chief for the lives of her children.", keynote: "Mother-instinct facing power." },
    SabianSymbol { sign: "Scorpio", degree: 30, image: "Children in Halloween costumes indulging in various pranks.", keynote: "Permitted reversal of order." },

    // ── SAGITTARIUS ───────────────────────────────────────────────
    SabianSymbol { sign: "Sagittarius", degree: 1,  image: "Retired army veterans gather to reawaken old memories.", keynote: "Shared past binding present." },
    SabianSymbol { sign: "Sagittarius", degree: 2,  image: "The ocean covered with whitecaps.", keynote: "Surface activity over depth." },
    SabianSymbol { sign: "Sagittarius", degree: 3,  image: "Two men playing chess.", keynote: "Mental contest within rules." },
    SabianSymbol { sign: "Sagittarius", degree: 4,  image: "A little child learning to walk.", keynote: "First independent steps." },
    SabianSymbol { sign: "Sagittarius", degree: 5,  image: "An old owl perches alone on the branch of a large tree.", keynote: "Wisdom in solitude." },
    SabianSymbol { sign: "Sagittarius", degree: 6,  image: "A game of cricket.", keynote: "Group sport of refined civility." },
    SabianSymbol { sign: "Sagittarius", degree: 7,  image: "Cupid knocks at the door of a human heart.", keynote: "Love announcing itself." },
    SabianSymbol { sign: "Sagittarius", degree: 8,  image: "Within the depths of the earth new elements are being formed.", keynote: "Hidden alchemy underway." },
    SabianSymbol { sign: "Sagittarius", degree: 9,  image: "A mother leads her small child step by step up the stairs.", keynote: "Patient guidance of growth." },
    SabianSymbol { sign: "Sagittarius", degree: 10, image: "A theatrical representation of a golden-haired goddess of opportunity.", keynote: "Fortune dramatized for the hopeful." },
    SabianSymbol { sign: "Sagittarius", degree: 11, image: "The lamp of physical enlightenment at the left temple.", keynote: "Concrete intellect awakened." },
    SabianSymbol { sign: "Sagittarius", degree: 12, image: "A flag turns into an eagle that crows.", keynote: "Symbol come alive." },
    SabianSymbol { sign: "Sagittarius", degree: 13, image: "A widow's past is brought to light.", keynote: "Hidden truths of one chapter exposed." },
    SabianSymbol { sign: "Sagittarius", degree: 14, image: "The Pyramids and the Sphinx.", keynote: "Eternal questions made monumental." },
    SabianSymbol { sign: "Sagittarius", degree: 15, image: "The ground hog looking for its shadow on Ground Hog Day.", keynote: "Self-reflection as oracle." },
    SabianSymbol { sign: "Sagittarius", degree: 16, image: "Sea gulls fly around a ship looking for food.", keynote: "Opportunism in flight." },
    SabianSymbol { sign: "Sagittarius", degree: 17, image: "An Easter sunrise service draws a large crowd.", keynote: "Communal celebration of resurrection." },
    SabianSymbol { sign: "Sagittarius", degree: 18, image: "Tiny children in sunbonnets.", keynote: "Innocence shielded from the bright sun." },
    SabianSymbol { sign: "Sagittarius", degree: 19, image: "Pelicans, disturbed by the garbage of people, move their young to a new habitat.", keynote: "Migration in response to disturbance." },
    SabianSymbol { sign: "Sagittarius", degree: 20, image: "In an old-fashioned northern village men cut the ice of a frozen pond, for use during the summer.", keynote: "Storing one season's gifts for another." },
    SabianSymbol { sign: "Sagittarius", degree: 21, image: "A child and a dog wearing borrowed eyeglasses.", keynote: "Borrowed vision yielding play." },
    SabianSymbol { sign: "Sagittarius", degree: 22, image: "A Chinese laundry.", keynote: "Modest service rendered with dignity." },
    SabianSymbol { sign: "Sagittarius", degree: 23, image: "A group of immigrants as they fulfill the requirements of entrance into the new country.", keynote: "Threshold of rebirth into a new life." },
    SabianSymbol { sign: "Sagittarius", degree: 24, image: "A bluebird standing at the door of the house.", keynote: "Happiness about to enter." },
    SabianSymbol { sign: "Sagittarius", degree: 25, image: "A chubby boy on a hobby-horse.", keynote: "Imaginative play imitating the grown world." },
    SabianSymbol { sign: "Sagittarius", degree: 26, image: "A flag-bearer in a battle.", keynote: "Courageous representation of cause." },
    SabianSymbol { sign: "Sagittarius", degree: 27, image: "A sculptor at his work.", keynote: "Vision shaping resistant material." },
    SabianSymbol { sign: "Sagittarius", degree: 28, image: "An old bridge over a beautiful stream is still in constant use.", keynote: "Ancient form serving present need." },
    SabianSymbol { sign: "Sagittarius", degree: 29, image: "A fat boy mowing the lawn of his house on an elegant suburban street.", keynote: "Routine effort within comfort." },
    SabianSymbol { sign: "Sagittarius", degree: 30, image: "The Pope, blessing the faithful.", keynote: "Sacred authority extending grace." },

    // ── CAPRICORN ─────────────────────────────────────────────────
    SabianSymbol { sign: "Capricorn", degree: 1,  image: "An Indian chief claims power from the assembled tribe.", keynote: "Authority drawn from group consent." },
    SabianSymbol { sign: "Capricorn", degree: 2,  image: "Three stained-glass windows in a Gothic church, one damaged by war.", keynote: "Sacred ideal scarred by conflict." },
    SabianSymbol { sign: "Capricorn", degree: 3,  image: "The human soul, in its eagerness for new experiences, seeks embodiment.", keynote: "Spirit yearning for incarnation." },
    SabianSymbol { sign: "Capricorn", degree: 4,  image: "A group of people entering a large canoe for a journey by water.", keynote: "Group commitment to crossing." },
    SabianSymbol { sign: "Capricorn", degree: 5,  image: "Indians on the warpath. While some men row a well-filled canoe, others in it perform a war dance.", keynote: "Mobilized aggression in motion." },
    SabianSymbol { sign: "Capricorn", degree: 6,  image: "Ten logs lie under an archway leading to darker woods.", keynote: "Ten thresholds before the unknown." },
    SabianSymbol { sign: "Capricorn", degree: 7,  image: "A veiled prophet of power.", keynote: "Hidden source of authority." },
    SabianSymbol { sign: "Capricorn", degree: 8,  image: "Birds in the house singing happily.", keynote: "Domestic joy amid practicality." },
    SabianSymbol { sign: "Capricorn", degree: 9,  image: "An angel carrying a harp.", keynote: "Pure tone transmitted from above." },
    SabianSymbol { sign: "Capricorn", degree: 10, image: "An albatross feeding from the hand of a sailor.", keynote: "Wild trust, bond at sea." },
    SabianSymbol { sign: "Capricorn", degree: 11, image: "A large group of pheasants on a private estate.", keynote: "Aristocratic abundance." },
    SabianSymbol { sign: "Capricorn", degree: 12, image: "An illustrated lecture on natural science reveals little-known aspects of life.", keynote: "Hidden pattern of nature taught." },
    SabianSymbol { sign: "Capricorn", degree: 13, image: "A fire worshiper meditates on the ultimate realities of existence.", keynote: "Inner flame as object of contemplation." },
    SabianSymbol { sign: "Capricorn", degree: 14, image: "An ancient bas-relief carved in granite remains a witness to a long-forgotten culture.", keynote: "Permanence of form outlasting culture." },
    SabianSymbol { sign: "Capricorn", degree: 15, image: "In a hospital, the children's ward is filled with toys.", keynote: "Care extended into joy." },
    SabianSymbol { sign: "Capricorn", degree: 16, image: "School grounds filled with boys and girls in gymnasium suits.", keynote: "Discipline of body in play." },
    SabianSymbol { sign: "Capricorn", degree: 17, image: "A repressed woman finds psychological release in nudism.", keynote: "Liberation from imposed constraint." },
    SabianSymbol { sign: "Capricorn", degree: 18, image: "The Union Jack flies from a new British warship.", keynote: "Tradition projected into modern force." },
    SabianSymbol { sign: "Capricorn", degree: 19, image: "A child of about five carrying a huge shopping bag filled with groceries.", keynote: "Innocent shoulders large duty." },
    SabianSymbol { sign: "Capricorn", degree: 20, image: "A hidden choir singing during a religious service.", keynote: "Unseen support amplifying ritual." },
    SabianSymbol { sign: "Capricorn", degree: 21, image: "A relay race.", keynote: "Continuity through cooperation." },
    SabianSymbol { sign: "Capricorn", degree: 22, image: "A general accepting defeat gracefully.", keynote: "Dignity in loss." },
    SabianSymbol { sign: "Capricorn", degree: 23, image: "A soldier receiving two awards for bravery in combat.", keynote: "Courage publicly recognized." },
    SabianSymbol { sign: "Capricorn", degree: 24, image: "A woman entering a convent.", keynote: "Worldly renunciation for sacred path." },
    SabianSymbol { sign: "Capricorn", degree: 25, image: "An oriental rug dealer in a store filled with precious ornamental rugs.", keynote: "Curator of patterned wealth." },
    SabianSymbol { sign: "Capricorn", degree: 26, image: "A nature spirit dancing in the iridescent mist of a waterfall.", keynote: "Spirit at the edge of form." },
    SabianSymbol { sign: "Capricorn", degree: 27, image: "A mountain pilgrimage.", keynote: "Climbing as spiritual practice." },
    SabianSymbol { sign: "Capricorn", degree: 28, image: "A large aviary.", keynote: "Diverse voices in shared structure." },
    SabianSymbol { sign: "Capricorn", degree: 29, image: "A woman reading tea leaves.", keynote: "Reading the small for the large." },
    SabianSymbol { sign: "Capricorn", degree: 30, image: "A secret meeting of men responsible for executive decisions in world affairs.", keynote: "Inner circle shaping outer reality." },

    // ── AQUARIUS ──────────────────────────────────────────────────
    SabianSymbol { sign: "Aquarius", degree: 1,  image: "An old adobe mission in California.", keynote: "Faith preserved in modest form." },
    SabianSymbol { sign: "Aquarius", degree: 2,  image: "An unexpected thunderstorm.", keynote: "Surprise force interrupting calm." },
    SabianSymbol { sign: "Aquarius", degree: 3,  image: "A deserter from the navy.", keynote: "Refusal of imposed direction." },
    SabianSymbol { sign: "Aquarius", degree: 4,  image: "A Hindu healer glows with a mystic healing power.", keynote: "Inner energy radiating outward." },
    SabianSymbol { sign: "Aquarius", degree: 5,  image: "A council of ancestors is seen implementing the efforts of a young leader.", keynote: "Heritage supporting the new." },
    SabianSymbol { sign: "Aquarius", degree: 6,  image: "A performer of a mystery play.", keynote: "Sacred ritual enacted for others." },
    SabianSymbol { sign: "Aquarius", degree: 7,  image: "A child is seen being born out of an egg.", keynote: "New life through unusual portal." },
    SabianSymbol { sign: "Aquarius", degree: 8,  image: "Beautifully gowned wax figures on display.", keynote: "Ideal forms exhibited." },
    SabianSymbol { sign: "Aquarius", degree: 9,  image: "A flag turns into an eagle that crows.", keynote: "Static symbol becoming dynamic." },
    SabianSymbol { sign: "Aquarius", degree: 10, image: "A popularity that proves to be fleeting.", keynote: "Public favor's impermanence." },
    SabianSymbol { sign: "Aquarius", degree: 11, image: "During a silent hour, a man receives a new inspiration.", keynote: "Quietude opening to revelation." },
    SabianSymbol { sign: "Aquarius", degree: 12, image: "On a vast staircase stand people of different types, graduated upwards.", keynote: "Social hierarchy as ladder of growth." },
    SabianSymbol { sign: "Aquarius", degree: 13, image: "A barometer.", keynote: "Sensitivity to atmospheric change." },
    SabianSymbol { sign: "Aquarius", degree: 14, image: "A train entering a tunnel.", keynote: "Forward motion through hidden passage." },
    SabianSymbol { sign: "Aquarius", degree: 15, image: "Two lovebirds sitting on a fence and singing happily.", keynote: "Joy in simple intimacy." },
    SabianSymbol { sign: "Aquarius", degree: 16, image: "A big-businessman at his desk.", keynote: "Power harnessed to organization." },
    SabianSymbol { sign: "Aquarius", degree: 17, image: "A watchdog standing guard, protecting his master and his possessions.", keynote: "Loyal vigilance." },
    SabianSymbol { sign: "Aquarius", degree: 18, image: "A man unmasked.", keynote: "Public truth revealed." },
    SabianSymbol { sign: "Aquarius", degree: 19, image: "A forest fire is being subdued by the use of water, chemicals and sheer muscular energy.", keynote: "Containment of catastrophic force." },
    SabianSymbol { sign: "Aquarius", degree: 20, image: "A large white dove bearing a message.", keynote: "Annunciation of peace." },
    SabianSymbol { sign: "Aquarius", degree: 21, image: "A woman disappointed and disillusioned, courageously facing a seemingly empty life.", keynote: "Strength after loss of meaning." },
    SabianSymbol { sign: "Aquarius", degree: 22, image: "A rug placed on a floor for children to play on.", keynote: "Defined safe space within larger world." },
    SabianSymbol { sign: "Aquarius", degree: 23, image: "A big bear sitting down and waving all its paws.", keynote: "Latent power expressing playfully." },
    SabianSymbol { sign: "Aquarius", degree: 24, image: "A man turning his back on his passions teaches deep wisdom from his experience.", keynote: "Mastery teaching mastery." },
    SabianSymbol { sign: "Aquarius", degree: 25, image: "A butterfly with the right wing more perfectly formed.", keynote: "Asymmetric grace." },
    SabianSymbol { sign: "Aquarius", degree: 26, image: "A garage man testing a car's battery with a hydrometer.", keynote: "Diagnostic measurement of life-force." },
    SabianSymbol { sign: "Aquarius", degree: 27, image: "An ancient pottery bowl filled with violets.", keynote: "Modest container holding fragile beauty." },
    SabianSymbol { sign: "Aquarius", degree: 28, image: "A tree felled and sawed to ensure a supply of wood for the winter.", keynote: "Foresighted preparation for hardship." },
    SabianSymbol { sign: "Aquarius", degree: 29, image: "A butterfly emerging from a chrysalis.", keynote: "Transformation into winged form." },
    SabianSymbol { sign: "Aquarius", degree: 30, image: "The field of Ardath in bloom.", keynote: "Mystical flowering of consciousness." },

    // ── PISCES ────────────────────────────────────────────────────
    SabianSymbol { sign: "Pisces", degree: 1,  image: "A public market.", keynote: "Common exchange of life essentials." },
    SabianSymbol { sign: "Pisces", degree: 2,  image: "A squirrel hiding from hunters.", keynote: "Instinctive self-preservation." },
    SabianSymbol { sign: "Pisces", degree: 3,  image: "A petrified forest.", keynote: "Organic life made permanent in mineral." },
    SabianSymbol { sign: "Pisces", degree: 4,  image: "Heavy car traffic on a narrow isthmus linking two seashore resorts.", keynote: "Pressure point connecting two realms." },
    SabianSymbol { sign: "Pisces", degree: 5,  image: "A church bazaar.", keynote: "Sacred and commercial intermixed." },
    SabianSymbol { sign: "Pisces", degree: 6,  image: "A parade of army officers in full dress.", keynote: "Disciplined power on display." },
    SabianSymbol { sign: "Pisces", degree: 7,  image: "Illumined by a shaft of light, a large cross lies on rocks surrounded by sea mist.", keynote: "Sacred symbol revealed in mystery." },
    SabianSymbol { sign: "Pisces", degree: 8,  image: "A girl blowing a bugle.", keynote: "Innocent call to action." },
    SabianSymbol { sign: "Pisces", degree: 9,  image: "A jockey spurs his horse to outdistance his rivals.", keynote: "Drive to surpass competitors." },
    SabianSymbol { sign: "Pisces", degree: 10, image: "An aviator pursues his journey, flying through ground-obscuring clouds.", keynote: "Confident progress through obscurity." },
    SabianSymbol { sign: "Pisces", degree: 11, image: "Men traveling a narrow path, seeking illumination.", keynote: "Many on the same focused way." },
    SabianSymbol { sign: "Pisces", degree: 12, image: "An examination of initiates in the sanctuary of an occult brotherhood.", keynote: "Test of initiation." },
    SabianSymbol { sign: "Pisces", degree: 13, image: "An ancient sword, used in many battles, is displayed in a museum.", keynote: "Warrior tool transformed into artifact." },
    SabianSymbol { sign: "Pisces", degree: 14, image: "A lady wrapped in a large stole of fox fur.", keynote: "Cunning power worn as ornament." },
    SabianSymbol { sign: "Pisces", degree: 15, image: "An officer drilling his men in a simulated attack.", keynote: "Practiced readiness." },
    SabianSymbol { sign: "Pisces", degree: 16, image: "In the quiet of his study, a creative individual experiences a flow of inspiration.", keynote: "Solitary receptivity to influx." },
    SabianSymbol { sign: "Pisces", degree: 17, image: "An Easter promenade.", keynote: "Public celebration of renewal." },
    SabianSymbol { sign: "Pisces", degree: 18, image: "In a huge tent a famous revivalist conducts his meeting with a spectacular performance.", keynote: "Mass conversion through theater." },
    SabianSymbol { sign: "Pisces", degree: 19, image: "A master instructing his disciple.", keynote: "One-to-one transmission of essence." },
    SabianSymbol { sign: "Pisces", degree: 20, image: "A table set for an evening meal.", keynote: "Shared sustenance prepared." },
    SabianSymbol { sign: "Pisces", degree: 21, image: "Under the watchful and kind eye of a Chinese servant, a girl fondles a little white lamb.", keynote: "Innocence cared for by tradition." },
    SabianSymbol { sign: "Pisces", degree: 22, image: "A prophet bringing down the new law from Mount Sinai.", keynote: "Divine principle made human law." },
    SabianSymbol { sign: "Pisces", degree: 23, image: "A 'Materializing Medium' giving a séance.", keynote: "Form summoned from formless." },
    SabianSymbol { sign: "Pisces", degree: 24, image: "An inhabited island.", keynote: "Self-sufficient bounded life." },
    SabianSymbol { sign: "Pisces", degree: 25, image: "The purging of the priesthood.", keynote: "Reform of sacred institution." },
    SabianSymbol { sign: "Pisces", degree: 26, image: "A new moon reveals that it is time for people to go ahead with their different projects.", keynote: "Subtle signal to begin." },
    SabianSymbol { sign: "Pisces", degree: 27, image: "A harvest moon illuminates a clear autumnal sky.", keynote: "Fullness lighting completion." },
    SabianSymbol { sign: "Pisces", degree: 28, image: "A fertile garden under the full moon.", keynote: "Lunar nurturance of growth." },
    SabianSymbol { sign: "Pisces", degree: 29, image: "Light breaking into many colors as it passes through a prism.", keynote: "Unity revealed as spectrum." },
    SabianSymbol { sign: "Pisces", degree: 30, image: "A majestic rock formation resembling a face is idealized by a boy who takes it as his ideal of greatness.", keynote: "Eternal form inspiring youth." },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Aries 0° → degree 1 (first symbol in Aries).
    #[test]
    fn aries_first_degree() {
        let s = sabian_for_longitude(0.0);
        assert_eq!(s.sign, "Aries");
        assert_eq!(s.degree, 1);
        assert!(s.image.contains("woman"));
    }

    /// Aries 0.5° still rounds to degree 1.
    #[test]
    fn aries_first_degree_half() {
        let s = sabian_for_longitude(0.5);
        assert_eq!(s.degree, 1);
    }

    /// Aries 1.0° → degree 2.
    #[test]
    fn aries_one_degree_into() {
        let s = sabian_for_longitude(1.0);
        assert_eq!(s.degree, 2);
    }

    /// AAPL natal Sun ~261° → Sagittarius 21° "A child and a dog wearing borrowed eyeglasses."
    #[test]
    fn aapl_natal_sun_sabian() {
        let s = sabian_for_longitude(260.5); // Sagittarius 20.5°
        assert_eq!(s.sign, "Sagittarius");
        assert_eq!(s.degree, 21);
        assert!(s.image.to_lowercase().contains("child") || s.image.to_lowercase().contains("eyeglasses"));
    }

    /// Pisces 30° edge — should not overflow.
    #[test]
    fn pisces_last_degree() {
        let s = sabian_for_longitude(359.99);
        assert_eq!(s.sign, "Pisces");
        assert_eq!(s.degree, 30);
    }

    /// Negative longitude wraps.
    #[test]
    fn negative_longitude_wraps() {
        let s = sabian_for_longitude(-30.0); // == 330° == Pisces 1°
        assert_eq!(s.sign, "Pisces");
        assert_eq!(s.degree, 1);
    }

    /// All 360 entries unique by (sign, degree).
    #[test]
    fn coverage_is_360() {
        let mut seen = std::collections::HashSet::new();
        for s in &SABIAN_SYMBOLS {
            assert!(seen.insert((s.sign, s.degree)));
        }
        assert_eq!(seen.len(), 360);
    }

    /// Label format renders cleanly.
    #[test]
    fn label_format() {
        let s = sabian_for_longitude(0.0);
        let label = sabian_label(s);
        assert!(label.contains("Aries 1°"));
        assert!(label.contains("woman"));
    }
}
