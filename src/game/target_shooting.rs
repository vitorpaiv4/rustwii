use crate::audio::{play_back, play_click, play_start_chime};
use crate::inertial::{CursorState, PLAYER_COLORS};
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetKind {
    Standard, // 100 pts
    Gold,     // 300 pts
    Bomb,     // -150 pts
}

#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub id: usize,
    pub kind: TargetKind,
    pub x: f64, // 0.0 to 100.0 %
    pub y: f64, // 0.0 to 100.0 %
    pub vx: f64,
    pub vy: f64,
    pub radius_pct: f64,
    pub alive: bool,
}

impl Target {
    pub fn new(id: usize, kind: TargetKind, x: f64, y: f64, vx: f64, vy: f64) -> Self {
        let radius_pct = match kind {
            TargetKind::Standard => 4.5,
            TargetKind::Gold => 3.2,
            TargetKind::Bomb => 4.0,
        };
        Self {
            id,
            kind,
            x,
            y,
            vx,
            vy,
            radius_pct,
            alive: true,
        }
    }

    pub fn update(&mut self, dt: f64) {
        if !self.alive {
            return;
        }

        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // Bounce on screen bounds
        if self.x < 5.0 || self.x > 95.0 {
            self.vx = -self.vx;
            self.x = self.x.clamp(5.0, 95.0);
        }
        if self.y < 12.0 || self.y > 82.0 {
            self.vy = -self.vy;
            self.y = self.y.clamp(12.0, 82.0);
        }
    }

    pub fn check_hit(&self, shot_x: f64, shot_y: f64) -> bool {
        if !self.alive {
            return false;
        }
        let dx = self.x - shot_x;
        let dy = self.y - shot_y;
        let dist = (dx * dx + dy * dy).sqrt();
        dist <= self.radius_pct
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatingEffect {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub text: String,
    pub color: &'static str,
    pub opacity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PlayerScore {
    pub score: i32,
    pub shots_fired: u32,
    pub hits: u32,
}

impl PlayerScore {
    pub fn accuracy(&self) -> f64 {
        if self.shots_fired == 0 {
            0.0
        } else {
            (self.hits as f64 / self.shots_fired as f64) * 100.0
        }
    }
}

#[component]
pub fn TargetShootingGame(
    cursors: Signal<[CursorState; 4]>,
    on_exit: EventHandler<()>,
) -> Element {
    let mut time_left = use_signal(|| 45); // 45 seconds match
    let mut game_over = use_signal(|| false);
    let mut player_scores = use_signal(|| [
        PlayerScore::default(),
        PlayerScore::default(),
        PlayerScore::default(),
        PlayerScore::default(),
    ]);

    let mut targets = use_signal(|| {
        vec![
            Target::new(1, TargetKind::Standard, 25.0, 30.0, 6.0, 4.0),
            Target::new(2, TargetKind::Standard, 75.0, 45.0, -7.0, 5.0),
            Target::new(3, TargetKind::Gold, 50.0, 60.0, 11.0, -8.0),
            Target::new(4, TargetKind::Bomb, 35.0, 70.0, -5.0, -6.0),
            Target::new(5, TargetKind::Standard, 65.0, 25.0, 8.0, 7.0),
        ]
    });

    let mut effects = use_signal(Vec::<FloatingEffect>::new);
    let mut next_effect_id = use_signal(|| 1usize);
    let mut last_click_state = use_signal(|| [false; 4]);

    // Game Timer Loop
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            spawn(async move {
                while *time_left.read() > 0 && !*game_over.read() {
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    let current = *time_left.read();
                    if current > 1 {
                        time_left.set(current - 1);
                    } else {
                        time_left.set(0);
                        game_over.set(true);
                        play_start_chime();
                    }
                }
            });
        }
    });

    // Motion & Animation Update Loop (30 FPS)
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            spawn(async move {
                while !*game_over.read() {
                    gloo_timers::future::TimeoutFuture::new(33).await;
                    let mut list = targets.write();
                    for t in list.iter_mut() {
                        t.update(0.033);
                    }

                    // Decay floating text effects
                    let mut eff = effects.write();
                    for e in eff.iter_mut() {
                        e.y -= 0.4;
                        e.opacity -= 0.04;
                    }
                    eff.retain(|e| e.opacity > 0.0);
                }
            });
        }
    });

    // Fire detection when player clicks button A or trigger B
    let current_cursors = cursors.read().clone();
    for (idx, cursor) in current_cursors.iter().enumerate() {
        let is_now_clicking = cursor.is_clicking || cursor.is_trigger;
        let was_clicking = last_click_state.read()[idx];

        if is_now_clicking && !was_clicking && cursor.is_active && !*game_over.read() {
            play_click();
            player_scores.write()[idx].shots_fired += 1;

            let shot_x = cursor.x;
            let shot_y = cursor.y;

            // Check hit against active targets
            let mut hit_target_id = None;
            let mut points = 0;
            let mut hit_color = PLAYER_COLORS[idx].primary;
            let mut text = "".to_string();

            {
                let mut target_list = targets.write();
                for t in target_list.iter_mut() {
                    if t.alive && t.check_hit(shot_x, shot_y) {
                        t.alive = false;
                        hit_target_id = Some(t.id);
                        match t.kind {
                            TargetKind::Standard => {
                                points = 100;
                                text = "+100".to_string();
                            }
                            TargetKind::Gold => {
                                points = 300;
                                text = "+300".to_string();
                            }
                            TargetKind::Bomb => {
                                points = -150;
                                hit_color = "#ef4444";
                                text = "-150".to_string();
                            }
                        }
                        break;
                    }
                }
            }

            if let Some(tid) = hit_target_id {
                player_scores.write()[idx].hits += 1;
                player_scores.write()[idx].score += points;

                // Spawn floating score effect
                let eid = *next_effect_id.read();
                next_effect_id.set(eid + 1);
                effects.write().push(FloatingEffect {
                    id: eid,
                    x: shot_x,
                    y: shot_y,
                    text,
                    color: hit_color,
                    opacity: 1.0,
                });

                // Respawn target after delay
                spawn(async move {
                    #[cfg(target_arch = "wasm32")]
                    gloo_timers::future::TimeoutFuture::new(1200).await;
                    let mut list = targets.write();
                    if let Some(t) = list.iter_mut().find(|t| t.id == tid) {
                        t.x = (10 + (tid * 17) % 80) as f64;
                        t.y = (15 + (tid * 23) % 65) as f64;
                        t.alive = true;
                    }
                });
            }
        }
    }

    last_click_state.set([
        current_cursors[0].is_clicking || current_cursors[0].is_trigger,
        current_cursors[1].is_clicking || current_cursors[1].is_trigger,
        current_cursors[2].is_clicking || current_cursors[2].is_trigger,
        current_cursors[3].is_clicking || current_cursors[3].is_trigger,
    ]);

    let scores = *player_scores.read();
    let target_list = targets.read().clone();
    let effect_list = effects.read().clone();

    rsx! {
        div {
            class: "wii-minigame-arena",

            // Top Game Header
            header {
                class: "game-top-hud",
                button {
                    class: "btn-game-exit",
                    onclick: move |_| {
                        play_back();
                        on_exit.call(());
                    },
                    "◀ Menu Wii"
                }

                div {
                    class: "game-timer-badge",
                    span { class: "timer-label", "TEMPO: " }
                    b { "{time_left}s" }
                }

                div {
                    class: "game-scores-row",
                    for (i, p) in scores.iter().enumerate() {
                        div {
                            class: "hud-score-pill",
                            style: "border-color: {PLAYER_COLORS[i].primary};",
                            span { style: "color: {PLAYER_COLORS[i].primary}; font-weight: bold;", "{PLAYER_COLORS[i].name}" }
                            b { "{p.score} pts" }
                        }
                    }
                }
            }

            // Target Playing Field
            div {
                class: "game-field",

                // Render Targets
                for target in target_list.iter() {
                    if target.alive {
                        div {
                            key: "{target.id}",
                            class: match target.kind {
                                TargetKind::Standard => "game-target target-standard",
                                TargetKind::Gold => "game-target target-gold",
                                TargetKind::Bomb => "game-target target-bomb",
                            },
                            style: "left: {target.x}%; top: {target.y}%; width: {target.radius_pct * 2.0}vw; height: {target.radius_pct * 2.0}vw;",
                            div { class: "target-inner" }
                        }
                    }
                }

                // Render Floating Score Popups
                for eff in effect_list.iter() {
                    div {
                        key: "{eff.id}",
                        class: "floating-score-popup",
                        style: "left: {eff.x}%; top: {eff.y}%; color: {eff.color}; opacity: {eff.opacity};",
                        "{eff.text}"
                    }
                }
            }

            // Game Over Summary Modal
            if *game_over.read() {
                div {
                    class: "wii-modal-backdrop",
                    div {
                        class: "game-over-card",
                        h1 { "Fim de Jogo" }
                        p { "Resultados da Partida de Tiro ao Alvo:" }

                        div {
                            class: "podium-scores-grid",
                            for (i, p) in scores.iter().enumerate() {
                                div {
                                    class: "podium-item",
                                    style: "border-color: {PLAYER_COLORS[i].primary};",
                                    h3 { style: "color: {PLAYER_COLORS[i].primary};", "{PLAYER_COLORS[i].name}" }
                                    p { class: "podium-score", "{p.score} Pontos" }
                                    span { class: "podium-acc", "Precisão: {p.accuracy():.1}% ({p.hits}/{p.shots_fired})" }
                                }
                            }
                        }

                        div {
                            class: "banner-action-row",
                            button {
                                class: "btn-wii-dialog-back",
                                onclick: move |_| {
                                    play_back();
                                    on_exit.call(());
                                },
                                "Voltar ao Menu Wii"
                            }
                            button {
                                class: "btn-wii-dialog-start",
                                onclick: move |_| {
                                    play_start_chime();
                                    time_left.set(45);
                                    game_over.set(false);
                                    player_scores.set([
                                        PlayerScore::default(),
                                        PlayerScore::default(),
                                        PlayerScore::default(),
                                        PlayerScore::default(),
                                    ]);
                                },
                                "Jogar Novamente"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_hit_detection() {
        let target = Target::new(1, TargetKind::Standard, 50.0, 50.0, 0.0, 0.0);
        // Direct hit at center
        assert!(target.check_hit(50.0, 50.0));
        // Hit within radius
        assert!(target.check_hit(52.0, 52.0));
        // Miss far away
        assert!(!target.check_hit(70.0, 70.0));
    }

    #[test]
    fn test_player_accuracy() {
        let mut stats = PlayerScore::default();
        assert_eq!(stats.accuracy(), 0.0);

        stats.shots_fired = 10;
        stats.hits = 7;
        assert_eq!(stats.accuracy(), 70.0);
    }

    #[test]
    fn test_target_movement_and_bounce() {
        let mut target = Target::new(1, TargetKind::Standard, 94.0, 50.0, 20.0, 0.0);
        target.update(0.1); // Moves past 95.0
        assert!(target.vx < 0.0); // Bounced to negative velocity
    }
}
