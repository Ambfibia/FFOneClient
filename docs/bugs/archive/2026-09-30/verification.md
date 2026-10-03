# История проверок

## Bug 34 / S62, S66, S68 — 2026-09-27

- `cargo test -p ffone-client production_catalog_resolves_known_primary_npc_routes --lib -- --nocapture` — 1 passed: NPC 3206 выбирает `npc_mandroid_chef`, другой Mandroid 2498 сохранил `npc_mandroid1`, Gold Banker 2586 и Morbucks Accountant 2587 выбирают HNPC 174 и 178.
- `cargo test -p ffone-client production_catalog_closes_all_published_hnpc_appearances --lib -- --nocapture` — 1 passed: каталог открывается, очки бухгалтера 178 используют отдельный GLB, другой носитель 183 сохраняет исходный.
- `target/debug/examples/hnpc_runtime_batch_probe.exe assets/game 178 178` — `HNPC appearance 178 ready in 6 frames`, 1 готовый вариант.
- `target/debug/examples/hnpc_player_gpu_preview.exe assets/game 174 OUTPUT.png 0` — GPU-кадр Gold Banker просмотрен: пиджак и брюки целые. Предпросмотр 178 до исправления показал отсутствие очков. Сравнение 183 с исходным `glasses_glasses3` и временно подставленным исправленным GLB показало оправу только во втором кадре; контрольный вариант 183 в каталоге возвращён к исходному пути.
- `target/debug/examples/logical_model_gpu_preview.exe --asset-root assets/game --model characters/shared/npc_mandroid1/npc_mandroid_chef.glb --screenshot OUTPUT.png --character-kind npc --true-root npc_mandroid_chef --npc-scale 1 --main-texture characters/shared/npc_mandroid1/npc_mandroid1.textures/npc_mandroid1.png` — `success`, 2 meshes/2 materials; кадр с шапкой просмотрен. Та же команда с `--animation stand1 --sample-midpoint true` — `success`, 69 кадров оценки анимации; шапка следует за головой.
- Адресный `hnpc_player_gpu_preview` для 178 из текущего исходного кода завершился тайм-аутом `Loading(ModularParts)` при уже привязанных частях; кадр бухгалтера после исправления этим способом не получен. Просмотр NPC 2586, 3206 и 2587 в игровой сессии не проводился.

## Bug 25 / S58 — 2026-09-27

- `cargo test -p ffone-client production_catalog_verifies_both_native_shared_rigs --lib -- --nocapture` — 1 passed, 0 failed.
- `cargo build -p ffone-client --example hnpc_player_gpu_preview --features diagnostics` — успешно.
- `target/debug/examples/hnpc_player_gpu_preview.exe assets/game 1 OUTPUT.png YAW_DEGREES` — мужской `m_face_001_type02`, 0°, 45°, 85°, 275°, 315°; `... assets/game 2 ...` — женский `f_face_001_type01`, те же углы. Все 9 снимков просмотрены: лицо видно. Выходы находятся во временном каталоге `bug025-{male,female}-*.png`.
- Проверка выполнена GPU-просмотром при дистанции 3.0; игровая сессия и неизвестный исходный ракурс S58 не проверены.

## Bug 14 / S76 — 2026-09-27

- `cargo check -p ffone-client --bin ffone-client` — успешно.
- `cargo test -p ffone-client --bin ffone-client app::gamepad::tests:: -- --nocapture` — 2 passed, 0 failed: профили, dead zone, фронт нажатия и сброс после отключения.
- `cargo test -p ffone-client --bin ffone-client pad_attack_obeys_modal_gate_and_releases -- --nocapture` — 1 passed, 0 failed: атака через Pad блокируется модальным gate и снимается при release.
- `cargo test -p ffone-client --bin ffone-client app::hotkeys::tests:: -- --nocapture` — 11 passed, 0 failed: затронутые маршруты действий и модальные блокировки.
- Физический геймпад на машине отсутствует; живой ввод, переназначение и навигация в клиенте не приняты. S76 остаётся активным.

При создании исходного аудита 2026-09-27 прошли 84 серверных и 33 клиентских адресных теста. Полный список команд и ограничения сохранены в [архиве](../2026-09-27/verification.md).

По следующему запросу пользователя очередь отфильтрована: исключены готовые исправления и задачи их повторной проверки. Сейчас активны 20 Bug и 25 исходных пунктов/нетронутых частей. Это изменение документации; игровые тесты повторно не запускались. Проверяется целостность ссылок, номеров и покрытия активных SNN.

## Bug 4 / S35 — 2026-09-27

- `cargo test -p ffone-client --bin ffone-client marquee_row_ -- --nocapture` — 2 passed, 0 failed. Проверены порядок STOP → warp 89/NPC 2248, координаты и блокировка движения после отправки.
- `cargo test -p ffone-client --lib normal_world_talk_gate_uses_exact_sight_range -- --nocapture` — 1 passed, 0 failed. Проверена локальная граница разговора по `m_iSightRange`.
- В `../RustyFusion`: `cargo test interaction_proximity_uses_exact_id_instance_and_inclusive_boundary -- --nocapture` — 1 passed, 0 failed. Проверены граница 800, дальний NPC и другой инстанс.
- Живой проход туда и обратно не выполнен: сервер не запущен, учётных данных для клиента нет. Это не подтверждение игрового сценария.

## Bug 16 / S16 (2026-09-27)

- `cargo run -p ffone-client --features diagnostics --example mission_dialogue_gpu_preview --locked` — успешно; новый `target/performance/menu-bugs/mission-long-ru.png` просмотрен: 328 px заголовок заполнен подложкой, прежний прямоугольник шириной 73 px исчез, длинная RU-строка и кнопка не пересекаются. Запуск через сервер не выполнялся.

## Bug 31 / S54 (2026-09-27)

- `cargo check -p ffone-client --lib --locked` — успешно.
- `cargo build -p ffone-client --features diagnostics --bin ffone-client --locked` — успешно.
- `FFONE_PERF_OUTPUT=target/bug-031/open-ground-moving FFONE_PERF_POSITION='-6365 -56.5 668' target/debug/ffone-client.exe` — успешно; `frame.png` просмотрен. С `FFONE_PERF_CONTACT_SHADOW_BASELINE=1` повторён тот же запуск: совпали поза камеры и положение игрока; участок земли у ног на кадре с тенью темнее.
- `FFONE_PERF_OUTPUT=target/bug-031/shadow FFONE_PERF_FREEZE=1 target/debug/ffone-client.exe` и тот же запуск с `FFONE_PERF_CONTACT_SHADOW_BASELINE=1`, вывод в `target/bug-031/baseline` — оба завершены; 600 интервалов, средние 12.748/12.585 мс, 669/668 видимых мешей соответственно. Один A/B замер, без вывода о стабильной разнице FPS.
- Мост и прыжок отдельным живым сценарием не проверены.



## Bug 32 / S08 — 2026-09-27

- `cargo test -p ffone-client --lib rendering::legacy_model_material::tests:: --locked` — 69 passed, 0 failed.
- `cargo build -p ffone-client --features diagnostics --example logical_model_gpu_preview --bin ffone-client --locked` — успешно.
- Сравнение исходных и изменённых GLB: BIN побайтово совпадает; JSON отличается только `nativeOutline` и cel Ван Клайса.

```powershell
foreach ($nanoName in @('mordecai','titan','vankleiss')) {
    foreach ($cameraView in @('primary','reverse')) {
        & target/debug/examples/logical_model_gpu_preview.exe --asset-root assets/game --model "characters/nanos/nano_$nanoName/nano_$nanoName.glb" --character-kind nano --true-root "nano_$nanoName" --animation stand1 --sample-midpoint true --camera-view $cameraView --screenshot "target/bug-032/$nanoName-$cameraView.png" --report "target/bug-032/$nanoName-$cameraView.json"
    }
}
foreach ($nanoId in @(64,57,66)) {
    $env:FFONE_PERF_OUTPUT="target/bug-032/client-$nanoId"
    $env:FFONE_PERF_NANO_ACQUISITION="$nanoId"
    & target/debug/ffone-client.exe
}
Remove-Item Env:FFONE_PERF_OUTPUT,Env:FFONE_PERF_NANO_ACQUISITION
```

Все шесть GPU-отчётов — success, без ошибок материалов/шейдеров. Все шесть кадров просмотрены. Три запуска основного клиента завершились успешно; `nano-acquisition.json` содержит `selected=true`, переходы PowerSelection → ResultSkill → Closed. Просмотрены `client-{64,57,66}/nano-<id>-PowerSelection.png`: контуры видны, прозрачность оболочки Титана сохранена, cel Ван Клайса включён. Это offline-приёмка в реальном приложении, не проверка сетевого сеанса.

## Bug 35 / S63 — 2026-09-27

- `cargo test -p ffone-client --lib effect_streaming_keeps_authored_nif_models_after_native_enqueue --locked --quiet` — 1 passed, 0 failed.
- `cargo build -p ffone-client --bin ffone-client --features diagnostics --locked --quiet` — успешно.
- `FFONE_PERF_OUTPUT=target/bug-035/after-hover FFONE_PERF_POSITION='-3324 -35 1785' FFONE_PERF_HOVER=1 FFONE_PERF_YAW=0 target/debug/ffone-client.exe` — exit 0; `frame.png` просмотрен в реальном клиенте на краю заражённой зоны. Одиночный кадр не доказывает вращение или повторный streaming.
- `FFONE_PERF_ENTRIES=2` с `FFONE_PERF_OUTPUT=target/bug-035/after`, `FFONE_PERF_POSITION='-3324 -37.6 1790'`, `FFONE_PERF_YAW=0` — первый кадр записан, затем exit 1 без второго кадра; повторный вход не принят.
