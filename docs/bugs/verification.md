# История проверок

## Bug 14 — дополнительная таблица: фокус, повтор и удержание, 2026-10-03

- `cargo test -p ffone-client --bin ffone-client gamepad -- --nocapture` — **17 passed**, 0 failed. Включены стабильное удержание, отпускание/повторный A, opt-in повтор стрелок, изоляция popup, первая опция NPC, Tutorial/chat ownership, вкладки и пушка (заряд/отпускание/отмена при отключении).
- Windows WGI `cargo check --manifest-path vendor/gilrs-core/Cargo.toml --target-dir target/bug-014-backend-check --message-format short` и сборка клиента — успешно.
- Полный клиент с синтетическим Bevy-геймпадом: `FFONE_PERF_OUTPUT=target/gamepad-validation/live`, `FFONE_PERF_GAMEPAD=1` — **BUG014 PASS, SCREENS PASS, JOURNAL PASS, FRONTEND PASS**, exit 0. Проверены D-pad/стик, A/B/Start, 5 reconnect; голубой предмет/крестик/миссия/категория; карточка A → B → A, B сохраняет инвентарь; LB/RB Items/Nanos и Active/Completed; два A на минус/плюс; выбор персонажа, создание и пропуск intro на A; повторный A рулетки и удержание стрелки причёсок. Снимки осмотрены. Лог: `target/gamepad-validation/live.log`.
- Подготовка offline fixture исправлена после двух ранних остановок: Error отсутствующего сервера больше не сбрасывает её инвентарь, NanoCom закрывается перед journal shortcut. Appearance переключается в модели без серверной регистрации имени, поэтому полный сетевой create этим не проверен.
- Актуальный bin-test executable, фильтр `equipment_refresh_tests` — **3 passed**: новое снаряжение и позиция при смене/создании персонажа, привязка к UID, обновление инвентаря/HUD/портретов.
- Физического геймпада нет (ответ пользователя). WGI USB/Bluetooth, серверные результаты и отдельные живые сценарии магазина/количества/Turbo/почты остаются непроверенными. [Bug 14](bug-014.md) остаётся активным.

## Bug 14 / T114–T122 — 2026-10-03

- `cargo check --manifest-path vendor/gilrs-core/Cargo.toml --target-dir target/bug-014-backend-check`, `cargo check -p ffone-client --bin ffone-client`, `cargo build -p ffone-client --bin ffone-client` — успешно.
- `cargo test -p ffone-client --bin ffone-client gamepad -- --nocapture` — 8 passed; новый случай проверяет A на зиплайне при доступной цели взаимодействия.
- Реальный клиент с offline fixture и синтетическим геймпадом (`FFONE_PERF_OUTPUT=target/bug-014-live-2026-10-03`, `FFONE_PERF_GAMEPAD=1`) — BUG014 PASS, exit 0: D-pad/левый стик, A/B в Options, Start через Enter reducer при открытом NanoCom gate, 5 циклов Bevy disconnect/reconnect. Снимок Controls осмотрен.
- Физический WGI USB/Bluetooth hot-plug, серверный Nano и отдельные окна инвентаря/Nano/наставника/транспорта не проверены; [Bug 14](bug-014.md) остаётся активным.

## Bug 74 / T140 — 2026-10-03

- В `../RustyFusion`: `cargo test routed_civilian_is_not_a_mob_target --lib` — 1 passed; моб type 59 не выбирает маршрутного мирного NPC type 3000, затем выбирает приблизившегося игрока.
- В `../RustyFusion`: `cargo test ai::tests --lib` — 13 passed, 1 failed. Новая проверка боевого союзника type 1013 и патрульного моба type 2568 прошла. Существующая `pack_followers_walk_with_their_roaming_leader` упала на конечной дистанции следования; отдельный повтор также упал.
- Исходные NPC ID и координаты T140 не представлены; реальный клиент и серверный раунд-трип по исходному эпизоду не проверены.

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

При создании исходного аудита 2026-09-27 прошли 84 серверных и 33 клиентских адресных теста. Полный список команд и ограничения сохранены в [архиве](archive/2026-09-27/verification.md).

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


## Bug 002 / T017, T026 — 2026-09-30

- `cargo test -p ffone-client-network --lib --locked --quiet` — 34 passed.
- `cargo test -p ffone-net --lib fake_openfusion --locked --quiet` — 9 passed, 21 filtered out.
- `cargo test -p ffone-client --bin ffone-client app::tests::character::state::character_selection_runtime_maps_one_based_slots_names_and_worldname --features diagnostics --locked --quiet` — 1 passed, 440 filtered out.
- `cargo build -p ffone-client --bin ffone-client --features diagnostics --locked --quiet` — успешно.
- Реальный клиент: `FFONE_CHARACTER_SESSION_PROBE_OUTPUT=target/bug-002/live-run2`, `FFONE_ISOLATED_TEST_DATABASE=1`; RustyFusion hybrid на 23420/23421 с `target/bug-002/session-run2.db`. В одном соединении созданы первый персонаж (UID 792667825134572255) и второй (UID 2957147781577211462), для каждого выполнены shard entry/CompleteTutorial и ChangeCharacter. `passed.txt` записан после финального screenshot; `selection-1.png` и `selection-final.png` просмотрены. Два персонажа доступны, оба с локацией «СЕКТОР В - БУДУЩЕЕ», следующий свободный слот — 3.
- Первый live-прогон остановился по ошибочному ожиданию THE SUBURBS в диагностике после успешного возврата настоящим кликом. Ожидание уточнено по существующему worldname lookup; повторный полный прогон прошёл. Обучающие миссии не проходились: проверялась граница сеансов с явным CompleteTutorial. Тестовый сервер остановлен; пользовательские данные не изменялись.

Полный результат: [архивная карточка](archive/2026-09-30/bug-002.md).

## 2026-09-30 — Bug 8: направление пушки и переходы аватара

- `target/debug/deps/ffone_client-d952fc4e5a1f90fb.exe ui::launcher::tests:: --nocapture`: 13 passed, включая направление камеры/выстрела на шести углах и подъём от vertical aim.
- `cargo test -p ffone-client --lib characters::avatar_action::tests --locked`: 30 passed, включая подтверждённую смену Hand, повторный Ready, ожидание renderer completion и удар → зиплайн → выход → новую атаку.
- `target/debug/deps/ffone_client-d952fc4e5a1f90fb.exe tutorial_runtime::player_rig_runtime::tests:: --nocapture`: 29 passed, включая Bevy replay/repeat, coalescing слоёв и восстановление основной анимации.
- `cargo build -p ffone-client --bin ffone-client --locked`: успешно.
- `git diff --check` по изменённым владельцам/карточке/реестру: успешно.

Живой сценарий не выполнен: локальный порт login 23000 не слушается, параметры тестового входа FFONE_LOGIN_ADDRESS/USERNAME/PASSWORD отсутствуют. Ни положение основания/ствола пушки, ни видимая смена оружия, ни реальный заезд на зиплайн не объявляются принятыми. Bug 8 оставлен активным для живой приёмки; подробности и контрольная пушка — [карточка](bug-008.md).

## 2026-10-01 — Bug 10 / T005: регистрация запросов барбера

- Новый `rustyfusion_barber_requests_accept_typed_wire_payloads_and_reject_wrong_sizes` до исправления: FAILED с `UnregisteredPacket { packet_type: 318767269 }`; после исправления: passed для open/confirm и отклонения размеров ±1.
- `cargo test -p ffone-protocol --lib registry:: -- --nocapture`: 5 passed.
- `cargo test -p ffone-client-network --lib registered -- --nocapture`: 1 passed, 33 filtered out.
- `cargo test -p ffone-net --lib gameplay_sender_barber -- --nocapture`: 1 passed, 30 filtered out; TCP-тест получает оба пакета с точными телами, шифрованием и checksum 0/1. В первом прогоне нового теста ожидался checksum 0 для обоих пакетов; исправлено ожидание последовательности в тесте.
- RustyFusion: `cargo test --locked --lib barber:: -- --test-threads=1`: 5 passed, включая подтверждение, стоимость, невалидные операции и сохранение/загрузку внешности, денег и одежды в изолированной SQLite БД.
- `cargo build -p ffone-client --bin ffone-client --locked`: успешно.
- `git diff --check` по затронутым владельцам и реестру: успешно.

Реальный клиент не проверен: процессы клиента/сервера отсутствуют, порт login 23000 не слушается, параметры FFONE_LOGIN_ADDRESS/FFONE_USERNAME/FFONE_PASSWORD отсутствуют. Конкретный пользовательский NPC/instance неизвестен. Серверная fixture — NPC 900347, type 3470. Снят как code-fixed, без полной живой приёмки: [карточка](archive/2026-10-01/bug-010.md).

## 2026-10-01 — Bug 25 / T056: диагностика, дефект не воспроизведён

- Прямой запуск lib-бинарника `target/debug/deps/ffone_client-d952fc4e5a1f90fb.exe` с фильтрами `skinned_frustum_policy_is_idempotent`, `skinned_meshes_disable_invalid_static_aabb_frustum_culling`, `production_catalog_verifies_both_native_shared_rigs`: по 1 passed, всего 3; нулевых прогонов нет.
- Старый GPU-бинарник `tutorial_player_rig_gpu_preview.exe` от 2026-09-19: `assets/game target/bug025-before.png 2 1 0 standup male`, `assets/game target/bug025-vehicle-old.png 2 1 0 vehicle-board male`, `assets/game target/bug025-female-vehicle-old.png 2 1 0 vehicle-board female` — exit 0. Кадры осмотрены: лицо видно. Транспортные прогоны подтвердили снятие attachment/trails. Выводилась ошибка каталога projectile 167. Это не проверка свежего исходного кода или ракурса из T056.
- `rustfmt --check --edition 2024 crates/ffone-client/examples/characters/tutorial_player_rig_gpu_preview/operations_setup.rs`: passed.
- Новая сборка диагностического просмотра сообщила ошибки текущего рабочего дерева: приватный `AnimatableProperty` и ненайденный `option_slider_thumb_left`. Новый обход ракурсов не выполнен.
- `git diff --check` по диагностическому файлу и карточке/README/source-list: passed.

T056 остаётся активным. Login 23000 не слушается, параметры тестового входа отсутствуют; живая приёмка не выполнена. [Подробности](bug-025.md).

## 2026-10-01 — Bug 13 / T018, T054, T055

- `cargo build -p ffone-client --bin ffone-client`: успешно.
- Собранный library test executable `target/debug/deps/ffone_client-c2ceebe582f346b8.exe ui::race::hud:: --nocapture`: 1 passed. Проверены готовность, возрастающее отображаемое время, секундное смещение после подтверждения и скрытие HUD.
- Дополнительный прогон того же executable с фильтром `race`: 29 passed, 2 failed. Неизменённые asset fixtures: `ui::race::mode::tests::every_result_texture_has_exact_source_hash_and_dimensions` (button_normal.png: актуальный SHA256 `35D939DB8C1E06F312AF256409672DB0613EAB01EBC0EC32FB12A9EBE7EAF999`, ожидается `6205BA81A517B3861976BCDC68B622F83E04C6A005B416F4CECA9862156ACCA5`) и `ui::race::rank::tests::all_semantic_rank_assets_exist_and_catalog_hash_is_exact` (актуальный `CDCB9F138FF0D45DA869B1DB66958E3D64FBD35AC8EEC245C460E6DEA2B253D9`, ожидается `9BEAAE20C635CD1F1D0CA4A2167B9BF052E3C3478738566D4B603CE0525D38E0`). Текстуры результатов/каталог рейтингов и эти тесты не изменялись; результаты сохранены, проверки не отключались.
- Реальный клиент `ffone-client.exe`, 1920×1080, RU, offline authority: `FFONE_PERF_RACE_PODS=1`, `FFONE_PERF_RACE_HUD_LIFECYCLE=1`, `FFONE_PERF_POSITION="-2933 -50.7 5467"`, `FFONE_PERF_ENTRIES=2`. Оба входа завершились успешно: видимы shell/core color passes, emission меняется на протяжении 60 кадров, подтверждённый подбор отображает 1 под, выход скрывает HUD с сохранёнными данными зоны. Под `(-2936.3145, -49.59778, 5469.9673)`, Динозаврий Проход. Лог: `target/performance/bug013-20261001/run.log`.
- Просмотрены `entry-1/race-100.png`, `race-160.png`, `race-355.png`, `frame.png`: время `00:10 → 00:11 → 00:14`, разные фазы свечения, смещение иконки после подбора, HUD/поды отсутствуют после выхода. Снимки лежат в `target/performance/bug013-20261001`.

Серверный старт/финиш и конкретная машина времени не проверены. Подбор/завершение в визуальном сценарии подставлены fixture через производственного владельца; полная серверная приёмка не заявлена. Результат: [Bug 13](archive/2026-10-01/bug-013.md).

Дополнение Bug 13: `cargo test -p ffone-client --lib ui::race:: -- --nocapture` завершился: 27 passed, 2 failed на тех же hash fixtures. Адресный HUD тест из актуального library executable `ffone_client-23d88effaca75c27.exe`: 1 passed.

Финальный прогон Bug 13: `cargo test -p ffone-client --bin ffone-client race -- --nocapture` — **13 passed, 0 failed**. Новые регрессии: `course_info_alone_does_not_show_ready_and_race_time_increases`, `glow_fades_and_repeats_without_blinking`, `pod_passes_pulse_without_changing_shared_materials`; прежние уникальные проверки корреляции, ring ABI, отмены, подтверждённого подбора и обычного instance notification также прошли. Ссылки архивной карточки и `git diff --check` по затронутым владельцам/реестру проверены.

## 2026-10-01 — Bug 14 / настройки и ввод геймпада

- `cargo check -p ffone-client --bin ffone-client`: успешно (до дополнительной правки Escape в Options).
- `cargo check --manifest-path vendor/gilrs-core/Cargo.toml --target-dir target/bug-014-backend-check`: успешно. Backend `cargo test --lib` содержит 0 тестов; это не приёмка hot-plug.
- `cargo test -p ffone-client --lib ui::option:: -- --nocapture`: **31 passed**, включая сохранение старых pad bindings и Apply/Cancel независимых настроек камеры.
- `cargo test -p ffone-client --lib controller_start_uses_enter_nanocom_reducer_once_per_press -- --nocapture`: **1 passed**.
- Library executable `target/debug/deps/ffone_client-23d88effaca75c27.exe round_trip_preserves_options_input_locales_and_selection_music --nocapture`: **1 passed**, новые pad settings сохранены и прочитаны с диска.
- Первый bin-прогон `gamepad`: 6 passed, 1 failed из-за недопустимого `Entity::from_bits` в новой тестовой fixture. Fixture заменена на `Entity::from_raw_u32`; повторный `cargo test -p ffone-client --bin ffone-client gamepad -- --nocapture`: **7 passed, 0 failed** (актуальный `ffone_client-58050a83d69fb95f.exe`).
- Первый живой offline-прогон `FFONE_PERF_OUTPUT=target/bug-014-live`, `FFONE_PERF_GAMEPAD=1`: D-pad перевёл фокус, A выбрал другую вкладку; B не закрыл Options. Найдена несовместимость обычного Escape и переназначаемого действия Escape (default BackQuote) в владельце Options; добавлена обработка обычного Escape. B также передаёт KeyboardInput для UI-окон, читающих сообщения, а не только ButtonInput. Повторная приёмка: свежий `target/debug/ffone-client.exe` от 09:32, RU 1920×1080 — **BUG014 PASS**, exit 0. D-pad/A/B и пять Bevy disconnect/reconnect циклов прошли; Quit после B не переоткрылся. Лог: `target/bug-014-live-direct.log`. Снимок `target/bug-014-live/bug-014-controls.png` осмотрен: новые pad controls видны.
- EN/RU JSON разобраны, оба новых ключа непустые. Адресный `git diff --check` прошёл.

Физический геймпад недоступен, что пользователь подтвердил. Пользовательский вылет и задержки Nano на сервере не воспроизведены. Синтетические события Bevy не заменяют физическую приёмку WGI/USB/Bluetooth. Карточка остаётся активной с узким остатком: [Bug 14](bug-014.md).

## 2026-10-01 — Bug 22: проваливание акторов и shared PNG

- `cargo test -p ffone-client --lib tutorial_runtime::actors::tests::collision:: --locked`: 5 passed. Повтор на свежем `target/debug/deps/ffone_client-300cdf94d89a9153.exe` после исправления материалов: 5 passed.
- `target/debug/deps/ffone_client-300cdf94d89a9153.exe rendering::legacy_model_material::tests:: --nocapture`: 70 passed. Новая регрессия использует реальные PNG/mip-хеши `_BumpMap` портала, сохраняет shared sRGB-источник, проверяет отдельные Linear/sRGB GPU handles и отказ на повреждённых пикселях. Лог `target/bug-022/material-tests.log`.
- Cargo-повтор `cargo test -p ffone-client --lib rendering::legacy_model_material::tests:: --locked`: 70 passed; `cargo test -p ffone-client --lib portal_shared_png_keeps_linear_mips --locked`: 1 passed. `cargo build -p ffone-client --bin ffone-client --locked`: успешно.
- Весь владелец tutorial actors: 43 passed, 2 failed. Обе ошибки старых ожиданий Dexter/type 2673 повторены на прежнем `ffone_client-d952fc4e5a1f90fb.exe`; маршруты/варианты не изменялись. Логи `target/bug-022/actors-tests.log`, `target/bug-022/baseline-asset-tests.log`.
- Полный клиент: `FFONE_PERF_OUTPUT=target/bug-022/live-tutorial-final`, `FFONE_PERF_TUTORIAL=1`, `FFONE_PERF_NPC_FLOOR=1`; exit 0. Два создания портала/type 2695 и моба/type 2674: оба материала ready, Y стабилен у terrain в обоих проходах. `npc-floor.json` записан, `frame.png` просмотрен: щупальца и моб видимы. До исправления в `live-tutorial-2.log` воспроизведена блокировка готовой сцены портала ошибкой Linear/sRGB `_BumpMap`; после исправления она отсутствует.
- Адресный `git diff --check` прошёл. FPS-результат не заявляется: одновременно выполнялись чужие сборки. Живая проверка offline, без серверного сеанса; выгрузка/возврат пола проверены ECS-регрессией.

Результат и ограничения: [архивная карточка Bug 22](archive/2026-10-01/bug-022.md). T051 снят из активной очереди как исправленный клиентский баг.

## 2026-10-01 — Bug 25: продолжение, порядок материалов после перепривязки

- Две новые регрессии `renderer_rebinding` на прежнем lib-бинарнике `ffone_client-300cdf94d89a9153.exe`: **2 FAILED** (старый индекс и несброшенное подтверждение). На исправленном `ffone_client-096954d8c41a6a13.exe` затронутый shared-assets набор: **11 passed**; ещё два теста позднего material companion и flattened renderer order: **2 passed**.
- `cargo build -p ffone-client --bin ffone-client --features diagnostics --locked` и аналогичная сборка `--example tutorial_player_rig_gpu_preview`: успешно.
- До runtime-исправления: оба пола, vehicle-board yaw 45/85/275/315°, 8 прогонов/16 осмотренных кадров; weapon-swap male 45/85° и female 45/275°, 4 прогона/8 осмотренных кадров. Все exit 0, лицо присутствует. После исправления: 8 vehicle-board прогонов exit 0, осмотрены 8 мужских кадров — лицо присутствует. Male weapon-swap 45/85° и female 45° завершены exit 0; оставшиеся кадры не приняты визуально.
- Свежий реальный клиент запущен с отдельным RustyFusion/SQLite и портами 23250/23251. Журнал: DX12, окно создано. Пользователь остановил Computer Use клавишей Escape; дальнейший UI-осмотр прекращён, тестовые клиент и сервер остановлены. Живая приёмка исходного T056 не подтверждена.

Bug 25 остаётся активным с частичным исправлением: [карточка](bug-025.md).


## 2026-10-01 — Bug 45 и Bug 49

- `cargo test -p ffone-client --lib ui::transportation::tests:: --locked`: **15 passed / 2 failed**. Новая проверка destination keys двух таблиц EN/RU прошла. Старые `clean_catalog_is_the_exact_manifest_owned_table_projection` (46,320,592 vs 43,132,658 bytes) и `all_semantic_pngs_match_their_clean_pathid_proofs_and_set_digest` (7b5187… vs 2f08ce…) падают одинаково на прежнем `target/debug/deps/ffone_client-d952fc4e5a1f90fb.exe` от 08:32. Таблица и PNG этой задачей не менялись.
- `target/debug/deps/ffone_client-300cdf94d89a9153.exe gameplay::guide::tests::`: **12 passed**. `cargo test -p ffone-client --lib ui::guide::tests:: --locked` после подключения описаний наставников: **18 passed**, включая новый EN/RU content lookup; сохранены уникальные проверки source geometry, fonts, input, request/reply correlation.
- `cargo test -p ffone-client --bin ffone-client session_lifecycle::tests:: --locked`: **2 passed**, регрессия включает same-poll metadata, возврат к roster с сохранённой login-сессией и действительное отключение без старого entitlement. `cargo test -p ffone-client --bin ffone-client npc_warp_level_gate --locked`: **1 passed**.
- На свежем `target/debug/deps/ffone_client-58050a83d69fb95f.exe`: `mentor_reply_is_transactional` и `malformed_or_mismatched_mentor_reply` — **2 passed**; `nano_create_commits_quest_bank_and_progression`, `nano_create_invalid_quest_slot` и `app::tests::nano_free_tuning::acquisition_` — **5 passed**. Нулевые наборы проверок не засчитаны.
- `node --test ../FusionForge/tools/native/test-localization-terminology.mjs`: **4 passed**, EN/RU key equality и multiplicities placeholders проверены на production bundles.
- Полный `ffone-client.exe`, RU 1920×1080, offline: предупреждение, переход к выбору Dexter, подтверждение/отмена, SystemMessage 111 — PASS, exit 0; `target/bug-045-049-live/ru/service-dialogue-pass.txt`, четыре PNG. Выбор и message 111 осмотрены. Эта UI-проверка не подтверждает серверный warp и сохранение после входа. Только opt-in `FFONE_PERF_SERVICE_DIALOGUE` отбрасывает события worker без shard-сессии, чтобы периодические запросы offline-режима не сбрасывали проверяемые окна; рабочий сетевой lifecycle не отключён.
- Полный клиент, RU: NPC type 964 на native [-3754.09,-55.90,4483.96] и СКАМПЕР type 2545 на [-5953.26,-57,767.63]; открытие, выбор destination 0, переведённые row name/region и subtitle — **PASS**, оба exit 0. `target/bug-049-transport-ru/transport-localized.png` и `target/bug-049-scamper-ru/transport-localized.png` осмотрены. Имя Пич-Крик видно в строке и выбранной подписи. Расположение/регистрация/стоимость и фон не менялись.
- RustyFusion round trip недоступен: локальные 23000/23001 не слушаются, FFONE_LOGIN_ADDRESS/FFONE_USERNAME/FFONE_PASSWORD не заданы. Подтверждение гида сервером, warp 76, persistence при повторном входе и серверный сценарий Nano/силовой фишки остаются неподтверждёнными. FPS-результаты не заявляются.

- Финальная `cargo build -p ffone-client --bin ffone-client --locked` успешна; exe от 2026-10-01 11:09:51. Повтор RU и EN на этой сборке: оба PASS, exit 0. Финальные кадры выбора/подтверждения RU и выбора EN осмотрены; четыре описания наставников соответствуют языку и помещаются в карточки. Всего уникальных адресных Rust-проверок: 55 passed, 2 прежних content expectations failed; терминология: 4 passed.
- Bug 45/T037 и Bug 49/T019,T028,T029,T030,T032 сняты из активной очереди как code-fixed; [Bug 45](archive/2026-10-01/bug-045.md), [Bug 49](archive/2026-10-01/bug-049.md). Серверная приёмка остаётся неподтверждённой.
- Финальный адресный `git diff --check` прошёл; относительные ссылки обеих архивных карточек проверены.


## 2026-10-01 — Bug 34 / T008

- Существующий `ffone_client-d952fc4e5a1f90fb.exe production_catalog_resolves_known_primary_npc_routes --nocapture`: **1 passed**, 2039 filtered; chef type 3206 и обычный Mandroid 2498 сохранили отдельные маршруты. Cargo-повтор отменён во время ожидания чужого build lock; нулевой набор из bin harness не засчитан.
- Сравнение GLB с LFS объектом HEAD: **PASS**, только TRS узла 50; BIN/геометрия/rig/клипы/материалы/texture refs идентичны, attachment остаётся под Head, его не перезаписывают animation channels.
- `target/debug/examples/logical_model_gpu_preview.exe --asset-root assets/game --model characters/shared/npc_mandroid1/npc_mandroid_chef.glb --character-kind npc --true-root npc_mandroid_chef --npc-scale 1 --camera-view reverse --main-texture characters/shared/npc_mandroid1/npc_mandroid1.textures/npc_mandroid1.png --screenshot target/bug-034/after-front.png --report target/bug-034/after-front.json`: **success**. Повтор с `--animation stand1 --sample-midpoint true` → `after-stand1`: **success**, 69 animation evaluation frames. Оба кадра осмотрены, 0 material/shader errors.
- Штатный icon index 721 → icon type 4 / number 1784, `icons/entities/npc/npcicon_1784.png` осмотрен: шапка сбоку; его не меняли.
- Полный клиент от 11:27, offline, `FFONE_PERF_NPCS=1`: native position [-3229.26,1.87,5290.97], затем [-3233,6,5291] с FREEZE; оба exit 0. Кадры `target/bug-034/client/frame.png` и `client-close/frame.png` осмотрены; геометрия/эффект не позволяют подтвердить точную посадку в игре. Интерактивный портрет и серверная приёмка не подтверждены. [Результат Bug 34](archive/2026-10-01/bug-034.md).

## 2026-10-01 — Bug 32: чёрная обводка обоих глаз Мордекая

- Nano 64 / `nano_mordecai`: исходный полный offline-клиент воспроизвёл слабую прерывистую границу белков при получении Nano и выборе силы.
- `ffone_client-300cdf94d89a9153.exe rendering::legacy_model_material::tests --nocapture`: **71 passed**, включая оба `native_nano_outlines`. Подтверждены ограничение ink rim обоими глазами, сохранение треугольников/winding, прежний outline и surface/mip/sampler contracts.
- Байтовое сравнение GLB: исходный BIN сохранён как префикс; nodes/skins/animations/images/textures/samplers и исходный материал тела совпадают. 120 глазных треугольников выделены в отдельный материал без изменений позиций/нормалей/UV/skin.
- `cargo build -p ffone-client --example logical_model_gpu_preview --bin ffone-client --features diagnostics --locked`: **PASS**. GPU preview на RTX 3060/Vulkan: `-z/stand1`, `reverse/skill1`, `-x/stand3`, midpoint — 3 осмотренных ракурса с чёрным краем глаз, сохранёнными белками/зрачками и контуром тела; materialErrors=0, shaderErrors=0, 2 skinned primitives, 28 анимаций, 2×9 mip levels.
- Первый фронтальный preview захватил только hull при status=success; он исключён из визуальных PASS. Повтор тех же параметров показал полную модель. Причина этого одиночного сбоя захвата не исследовалась в рамках Bug 32.
- Полный клиент: exe 11:39:40 и финальный exe 11:49:16, оба exit 0; `CameraApproach`/`call`, `PowerSelection`/`stand1`, `ResultSkill`/`skill1`, `Closed`, selected=true. Оба глаза очерчены чёрным без закрашивания белков и без замеченных depth-артефактов. Offline fixture подставляет ответ выбора силы; серверное получение Nano не проверено. FPS не заявляется из-за одновременных чужих сборок.
- Кадры сохранены в `evidence/2026-10-01/bug-032/`. Адресный `git diff --check` прошёл. T057 закрыт; [карточка Bug 32](archive/2026-10-01/bug-032.md) перенесена в архив, активный индекс обновлён.

## 2026-10-01 — Bug 33

- `cargo test -p ffone-client --lib world::location_presentation_tests:: --locked`: **4 passed**. Проверены асинхронный террейн/бюджетная трава, ожидание её текстуры, отсутствие травы без deadlock, готовность соседней воды и сброс при переносе позиции.
- Финальный `target/debug/deps/ffone_client-300cdf94d89a9153.exe world::`: **136 passed / 6 failed / 2 ignored**. Все восемь сохранённых readiness/reveal проверок прошли после добавления необходимых ресурсов тестовым App. Шесть остальных ошибок отдельно воспроизведены на прежнем `ffone_client-d952fc4e5a1f90fb.exe` от 08:32: Foster animation lookup, registry drift expectation, v2 catalog fixture, overflow census 560 vs 562, frozen root chain, terrain publisher smoke. Последние два ссылаются на отсутствующие fixture paths; тесты и данные не переписывались под PASS.
- `cargo build -p ffone-client --bin ffone-client --features diagnostics --locked`: **PASS**. Финальный exe от 12:03:54 (Москва) скопирован в `target/bug-033/ffone-client.exe`, чтобы чужая последующая сборка не подменила проверяемый бинарник.
- Полный RU-клиент: City Hall `-1845.7359 -57.24495 2720.1978`, `FFONE_PERF_ENTRIES=2`, первый вход нового процесса и повтор через CharacterSelect — **exit 0**, два финальных кадра осмотрены. Первый вход не означает очистку файлового кэша ОС.
- Полный RU-клиент, `FFONE_PERF_LOADING_WARP="-2933 -50.7 5467"`: production teleport City Hall → Динозаврий Проход — **exit 0**. Origin Loading→Ready→admitted; destination Loading→Ready→admitted. Runtime assertions подтвердили видимый loader, выключенное движение во время загрузки, готовность текущей позиции до закрытия loader и последующее включение движения. Loader и финальный кадр осмотрены.
- Кадры, JSON assertions и readiness log сохранены в [evidence/2026-10-01/bug-033](evidence/2026-10-01/bug-033/). Offline replay не подтверждает реальный ответ RustyFusion/серверный loading-complete acknowledgment. FPS не заявляется из-за параллельных чужих сборок/клиентов.
- T050/T053 сняты как code-fixed; [Bug 33](archive/2026-10-01/bug-033.md) перенесён в архив. Адресный `git diff --check` и относительные ссылки карточки проверены.

## 2026-10-01 — Bug 30: Downtown и сброс музыки заражённых зон

- Последний локальный Retrobution: `builds/retrobution-20260821`. Адресные `fusionforge inspect`/`dump-object-evidence`: `PastMusic.resourceFile`, AudioClip 8, `05 Midtown`; SHA-256 OGG `6fe7a7e168933b6bbcac4984bd79cdaf9bf6f4389cbc60c2dbf972c9ff7e6269`, совпадает с `assets/game/audio/music/05_midtown.ogg`. `Midtown V1` (RetroMusic.resourceFile, AudioClip 7) тоже совпадает с native файлом: `86c13c69ceb2bbde8caf569be6f4aed3d916dc0d8453a1401a5cb00a45cd8de3`. Одинаковые файлы повторно не записывались. Для обычного Downtown выбран `05 Midtown`; пользовательская оценка выбора композиции не подтверждена.
- `target/debug/deps/ffone_client-300cdf94d89a9153.exe world_systems::audio::tests:: --nocapture`: **7 passed, 0 failed**. Включены новая проверка четырёх Midtown-полигонов 60–63 при cooldown/выходе из всех зон, прежние warp/loading/lair и порядок полигонов, а также сохранение текущего источника на соседних участках с одним клипом. Первоначальный фильтр `world_audio::tests` дал 0 тестов и не использовался как доказательство.
- `cargo build -p ffone-client --bin ffone-client --features diagnostics --locked`: **PASS**. Финальный проверенный бинарник от 2026-10-01 12:33:52. Промежуточные сборки были заблокированы отсутствующим `MovementIntentQueue` в параллельной правке `player_face.rs`; финальная сборка прошла после исправления у владельца, изменение Bug 30 там не оставлено.
- Полный клиент: `FFONE_PERF_OUTPUT=target/performance/bug030-audio-regression`, `FFONE_PERF_POSITION="-2304 -50 2304"`, `FFONE_PERF_AUDIO=1`, `RUST_LOG=info,ffone_client::world_systems::audio=debug`; часы 1/60 s, без `FFONE_PERF_FREEZE`. **Exit 0, 7/7 этапов passed**: Midtown (61) → Pokey Oaks Junior High (4) → Midtown → Mandark's House (5) → Midtown → Time_machine (102) → Midtown. Реальные выходные `AudioSink` подтверждены для каждого трека; одновременно активно максимум одно world music audio. Возвраты Midtown выполнялись до истечения его 45 s cooldown. Фактические результаты: `target/performance/bug030-audio-regression/audio.json`, подробности — `runtime.log`.
- Первый аудиопрогон был остановлен на ошибочной точке сценария: центр всего составного полигона Time_machine находился вне зон. Исправлена только точка probe на центр первой комнаты; production-полигон не менялся. Обычные offline performance captures не доказывали аудиопереход: сетевые Error events без shard выдавали `stop`. Аудио-fixture изолированно отбрасывает эти события; обычная обработка сети сохранена.
- Ограничение: переходы координат/instance-состояния синтетические, звук и клиент настоящие. Серверный проход порталов и пользовательская оценка композиции не проверены. T002/T045 сняты как code-fixed: [карточка Bug 30](archive/2026-10-01/bug-030.md).

## 2026-10-01 — Bug 25: полная сцена и надевание/снятие

- Завершён осмотр всех 24 парных GPU-кадров после sort-order исправления: оба пола на ховерборде/после выхода и при weapon-swap; лицо присутствует.
- Финальный `cargo build -p ffone-client --bin ffone-client --features diagnostics --locked` успешен. Полный клиент DX12 / South Pokey Oaks, `FFONE_PERF_PLAYER_FACE=1`, gender 1 и 2: оба **exit 0**, по **120 кадров**, финальный PASS-маркер в обоих журналах.
- 10 состояний × 12 ракурсов: исходное, оружие 328/43, снятие оружия, ховерборд/выход, Ace Jacket, Jacket + Deer Headdress, снятие Head, снятие UpperBody в пустой слот. Проверены attachments/анимации/завершение кандидата экипировки. Все **240 кадров визуально осмотрены**, лицо присутствует; предметы фактически появляются и снимаются.
- Артефакты: `target/bug025-full-empty-{1,2}` и журналы `.log`/`.err.log`, QA-листы `target/bug025-empty-qa`. Это offline authority fixture с локальным `PcLoadData0104`; UI-клики и серверный обмен не проверены. Исходное исчезновение и связь найденного sort-order дефекта с ним не доказаны. Bug 25 остаётся активным, [подробности](bug-025.md).

## 2026-10-01 — Bug 42: leash нового боя и составы групп

- Воспроизведение T038: NPC type 59, runtime ID 2000000920, возврат после выхода за combat range 4000, новый бой на 9000 единиц дальше исходного спавна. В изолированной копии возврат старой строки `local spawn = npc:spawn_position()` даёт **FAIL** на первом новом ударе: `new hit triggered another reset` (`target/bug042-verify-6986018b/red-test.log` в RustyFusion).
- Исправленный `ai::tests`: **12 passed** (`../RustyFusion/target/bug042-ai-isolated.log`). Проверены три повторных удара, последующий штатный leash/ReturnHomeHeal, очистка combat_origin, ранее существовавшие тесты reset/HP, патрулей и pack follow. Первоначальная поздняя проверка координат теста захватывала уже возобновлённый roaming: заменена проверкой позиции пакета ReturnHomeHeal. Один первый прогон существующего pack-follow теста дал ошибку дистанции; отдельно и в следующем полном адресном прогоне он passed.
- T040: данные mobs.json совпадают с локальным OpenFusion по SHA256. Тест фактического `make_group_npcs` + EntityMap/FFClient queue проверил group 0 (type 174, 3 NPC) и group 27 (type 521 + 4 × 133): координаты, offsets, связь с лидером, AI и 3/5 NPC_ENTER. 364 тройки и 6 групп с четырьмя спутниками в принятых данных сохранены.
- Проверка проведена в `../RustyFusion/target/bug042-verify-6986018b`, поскольку рабочее дерево менялось параллельно и временно не собиралось из-за незавершённых trade/escort. В копии отключена только посторонняя незавершённая проверка `m_iCSUDEPNPCFollow`; исходник этой проверки не менялся. Реальный клиент не запускался; T038 code-fixed, T040 остаётся активным для живой приёмки. [Карточка](bug-042.md).
- Независимый target `../RustyFusion/target/bug042-build`: сборка изолированной копии successful, AI **12 passed** (`target/bug042-ai-independent.log`). Из-за общего пути take_damage/acquire_target проверены damage **9 passed**, CC **12 passed**, corruption **15 passed** с cwd той же копии и её Luau; всего **48 passed**. Логи `{damage,cc,corruption}-final.log` внутри копии. `git diff --check` для затронутых серверных файлов passed.

## 2026-10-01 — Bug 48: транспортные значки и регистрация

- `git diff --check` по семи затронутым Rust-файлам — exit 0.
- `rustc --edition 2024 --test target/bug-048/source-regressions.rs -o target/bug-048/source-regressions.exe`; `source-regressions.exe --nocapture` — **4 passed, 0 failed**. Harness извлекает pure production code и текущие регрессии без Bevy/network dependencies. Первоначальный тестовый helper переполнял сдвиг для 128; исправлен тестовый случай на 127, повторный запуск passed.
- Cargo-тесты новой регистрации, mission_indicators и reply lease не завершились: ожидали общий build lock, остановлены без результата. Diagnostics-сборка также ожидала очередь. Свежий полный клиент и RustyFusion-перезаход не приняты. [Code-fixed результат и ограничения](archive/2026-10-01/bug-048.md).


## 2026-10-01 — Bug 41 / T036: серверное сопровождение NPC

- RustyFusion: cargo test --lib escort -- --test-threads=1 — 4 passed. Packet handlers GROUP_INVITE, TASK_START, NPC_MOVE и TASK_END для NPC type 2567, задач 5209 → 5211, mission 840, map 143. До запуска миссии группа не двигает NPC; отставание запрещает END; после прибытия этап завершается. БД не использовалась.
- Проверены отмена, смена instance, скачок позиции и восстановление сохранённого авторского маршрута; classification delivery/defence сверена с локальным OpenFusion.
- cargo test --lib ai::tests:: -- --test-threads=1 — 12 passed. cargo check --lib --locked RustyFusion и адресные git diff --check успешны.
- Клиентский cargo test -p ffone-client --lib escort_requests_resolve_live_owners --locked остановлен после длительного ожидания занятой общей Cargo-очереди; выполнения теста не было. Живая приёмка конкретной миссии в FFOneClient не выполнена. Не утверждается TCP/shard или визуальное воспроизведение исходной жалобы.
- cargo build --bin hybrid --locked RustyFusion — успешно, серверный исполняемый файл собран. Сервер с пользовательской БД не запускался.
- T036 снят как code-fixed: [Bug 41](archive/2026-10-01/bug-041.md).

## Bug 44 — серверная ходьба, 2026-10-01

- `cargo test --lib ai::tests:: --locked -- --test-threads=1` в `../RustyFusion`: **12 passed**, 0 failed (финальный запуск). Проверены исходящие `P_FE2CL_NPC_MOVE` мирного NPC type 59: `iMoveStyle=0`, `iSpeed=300`; для мирного последователя — `iMoveStyle=0`, скорость не выше 300. Проверки работают через реальный Luau AI и серверную рассылку в тестовый клиент.
- Начальный запуск: 11 passed, 1 failed (`patrol_mob_can_be_hit_again_far_from_its_original_spawn`); финальный повтор прошёл. Промежуточная попытка сборки блокировалась текущими изменениями других задач (`trade::tests`, visibility escort, `load_code_items`), которые здесь не исправлялись.
- Это серверный тест рассылки, не подключение игрового клиента к shard. Клиентские проверки записываются отдельно после выполнения.

## 2026-10-01 — Bug 46: сохранение выбранного квеста

- Библиотека с исправлением собрана при cargo build -p ffone-client --bin ffone-client; сборка executable завершилась сторонними ошибками PC2PC/network и email/Nano HUD probes.
- target/bug-046-journal-selection-tests.exe --nocapture: **3 passed, 0 failed**; integration-тест скомпилирован напрямую через rustc --test против свежего libffone_client-f67341a256be7209.rlib (21:01). Проверены повторное открытие, перестановка/удаление квестов, TaskStop fallback, пустой список, история, категории и NPC-награда.
- cargo test -p ffone-client --lib ui::mission::tests -- --nocapture не дошёл до запуска из-за сторонней ошибки отсутствующей константы P_FE2CL_PC_ENTER.
- git diff --check затронутого runtime/существующего теста прошёл. Полный клиент и offline replay не запущены; визуальная и серверная приёмка не подтверждены.
- [Карточка Bug 46](archive/2026-10-01/bug-046.md).

## Bug 51 — слои popup Enter, 2026-10-01

- Изолированная проверка библиотеки `rustc --emit=metadata` с текущими аргументами Cargo и окружением пакета: **exit 0** (`target/bug-051/check/errors.log`).
- Проверка metadata бинарника: **exit 1**, четыре ошибки вне Bug 51 в `codec_handle_gameplay_frame.rs:195`, `email_regression.rs:134,136`, `nano_hud.rs:28` (`target/bug-051/check/bin-errors.log`).
- Cargo build и адресные chat interaction tests ожидали блокировки target, ожидание остановлено; тесты не были выполнены. Новый full-client сценарий `FFONE_PERF_QUICK_CHAT=1` не запускался, снимков нет. T031 — code-fixed с неподтверждённой живой приёмкой: [карточка](archive/2026-10-01/bug-051.md).


## 2026-10-01 — Bug 47: дистанция квестового NPC

- Native TableData: миссия 520, задачи 598–600, NPC 1088–1090, sight range 1500; RustyFusion task_end применяет RANGE_INTERACT 800.
- Отдельный rustc harness с текущими targeting/mod.rs и tests.rs и cached workspace-зависимостями: **11 passed / 0 failed**; `target/bug047-targeting.log`. Проверены TalkNpc после клика на 7.99/8.00/8.02/12 units, вертикальная дистанция, повторная проверка корней и invalid positions; существующие LOS/sight/remote targeting регрессии также прошли.
- Адресный git diff --check прошёл. Cargo-тест остановился на 6 ошибках параллельных pc2pc/remote изменений. Cargo check проверил библиотеку, но бинарник заблокирован E0277 в network_ingress/codec_handle_gameplay_frame.rs:195, вне Bug 47. Diagnostics build остановлен после длительного ожидания lock; full-client replay и серверное завершение задачи не проверены. [Карточка и точные ограничения](archive/2026-10-01/bug-047.md).

## 2026-10-01 — Bug 43: приоритет и общий cooldown NPC-текста

- Сохранена свежая тестовая библиотека `target/debug/deps/ffone_client-300cdf94d89a9153.exe` (21:05:54 Москва) как `target/bug-043/npc-speech-tests.exe`.
- `npc-speech-tests.exe ui::gameplay::tests::npc_speech_priority --nocapture`: **3 passed**. Очередь приветствия/фонового текста немедленно заменяется квестом; обязательный текст повторяется даже при совпадении с cooldown; два NPC/две строки таблицы с одинаковым исходным текстом блокируются до 599.999 s и разрешаются при 600 s; отключение пузырей сохраняет ограничение фоновой истории.
- Дополнительно выполнены два существующих соседних тестовых модуля: `localization_mission_dialogue_publishes_local` (**2 passed, 1 failed**) и `operations_computress_2555_autonomous_barker_uses_produ` (**15 passed, 1 failed**). Проверки приветствия, квестовой истории без пузыря, RU/EN речи Buttercup, фоновой строки Computress 2555/row 51 и границ пузыря прошли. Два падения вне изменённого поведения: `combat_target_localization_key_tracks_the_current_npc_type` ожидает «Нефтяной огр», данные дают «Нефтяной Огр»; `every_gameplay_hud_text_entity_has_semantic_ownership` обнаруживает HUD-текст без LocalizedText. Эти группы не объявлены полностью прошедшими.
- Первые попытки Cargo были заблокированы параллельными изменениями: приватные типы HUD/PC fade, неверная константа PC_ENTER, отсутствующая сущность outline в фикстуре, несовместимый вызов API обмена и TextFont. Исправлены точечные ошибки видимости, две тестовые фикстуры и аргумент capabilities; остальные правки владельцев сохранены. После появления готовой библиотеки выбранные тесты выполнены напрямую, оставшийся дублирующий запрос Cargo отменён.

## 2026-10-01 — Bug 54 / T024: адресные регрессии

- Свежий test harness target/debug/deps/ffone_client-300cdf94d89a9153.exe, сборка 21:05: npc_bubble_visibility — **1 passed**; npc_speech_priority — **3 passed**; authored_segment_hit_is_two_sided_and_returns_the_first_crossing — **1 passed**. Только адресные фильтры, глобальный набор не запускался.
- Первоначальный Cargo-запуск остановился до тестов на сторонних ошибках UI обмена; результат успешных проверок взят из свежего harness после их устранения. Живая приёмка на этом этапе ещё не подтверждена.


### Bug 44 — клиентские регрессии, 2026-10-01

- `cargo test -p ffone-client --lib hnpc_idle_preserves --locked`: **1 passed**. HNPC rifleguard сохраняет финальные позы до полного цикла, затем допускает cheer и уступает walk/death.
- Только что собранный этим запуском `target/debug/deps/ffone_client-300cdf94d89a9153.exe npc_bad_max --test-threads=1`: **9 passed**, 0 failed. Реальные GLB, AnimationPlayer и фиксированное 1/60 время: новый полный idle-цикл и walk/run без перемотки, существующие combat/end/sound/reset проверки.
- Первая попытка `cargo test -p ffone-client --lib npc_bad_max --locked` не собрала тесты из-за промежуточных правок `ui/pc2pc/gestures.rs` (capabilities аргумент и Bevy font types). Ошибки исправлены другой задачей, последующая компиляция и перечисленные тесты успешны. Общие compiler warnings не исправлялись в Bug 44.


## Bug 40 — 2026-10-01

Удалённый PC: интерполяция/animation requests, server attack broadcast, HP/level и переход видимости. PC ID 17 в регрессиях. `cargo test -p ffone-client --lib world_systems::remote:: --locked`: 9 passed. Собранный test binary `target/debug/deps/ffone_client-300cdf94d89a9153.exe` дал: lifecycle — 35 passed/1 failed; pc_visibility — 2 passed; pc_animation_regression — 1 passed; player_health_tests — 1 passed; remote_pc_ — 8 passed. Всего 56 уникальных passed; отдельно повторённый codec-тест не посчитан дважды. Подтверждены приватные fill/outline, восстановление pass, удаление дочерних сущностей, немедленное освобождение ID, повторное появление и cleanup session.

Сбой неизменённой `path_npc_move_stream_advances_visible_root_and_holds_during_pause` вызван двойной регистрацией `advance_network_npc_motion_0104` в fixture; Bevy отклоняет SystemTypeSet при создании расписания. Обе регистрации есть в HEAD, тест сохранён. `git diff --check` затронутых Rust-файлов прошёл.

Живая приёмка двух клиентов не выполнена: ffone-client/RustyFusion отсутствуют, login/shard 23000/23001 не слушаются, FFONE_LOGIN_ADDRESS/FFONE_USERNAME/FFONE_PASSWORD не заданы. Runtime ID и локация пользователя не подтверждены; компонентные проверки не заменяют AOI/движение/атаку/эмоут на сервере. [Карточка и ограничения](archive/2026-10-01/bug-040.md).

Bug 40: свежий полный `target/debug/ffone-client.exe` (mtime 2026-10-01 21:16) запущен обычным режимом на 15 секунд. Процесс оставался работающим, затем остановлен; panic/ошибок полного расписания в stdout/stderr нет, есть warning о defaults input settings. Визуальная приёмка удалённого PC и два серверных клиента этим запуском не подтверждены.

Контрольный `cargo build -p ffone-client --bin ffone-client --locked` для Bug 40 остановлен на ожидании общего build lock после проверки свежего exe от параллельной сборки; успешный результат этой команды не заявляется.

## 2026-10-01 — Bug 50 / T020

- Смена sibling order в production email list: inactive fill → Player → list rim → Guide → captions. ECS-регрессия проверяет порядок и normal/active/hover изображения.
- Production library test-exe от 21:05, `ui::email::tests::`: **33 passed**. Независимая сборка текущего email preview main через `rustc --test` с готовыми зависимостями: тот же набор **33 passed**.
- Полный production app в временной диагностической сборке `target/bug-050-client.exe`, `FFONE_PERF_EMAIL=1 FFONE_PERF_EMAIL_TABS=1`, `--language ru` / `--language en`: **exit 0** для обоих языков. Runtime assertions folder и ComputedStackIndex прошли для шести состояний на язык; все 12 GPU-кадров осмотрены. Слой Player не меняется при переключении/hover, подписи видимы, нижний край закрыт рамкой.
- Сборка связана с production library от 21:01 и её точными dependency fingerprints; временный entry-point добавляет текущий PC-distance helper, отсутствовавший в готовой библиотеке. Основной app/UI/asset код не подменён. Штатная Cargo-сборка текущего дерева не подтверждена из-за параллельной очереди/изменений.
- Физический указатель не проверен: Windows helper дважды вернул `foreground window did not report a process id`. Обычная NanoCom offline-fixture требует authoritative inventory и не принята; отдельный preview timed out на проверке FROM. Layer fixture открывает пустой mailbox через production open_email_ui и переключает production switch_email_folder; серверные письма не проверялись.
- `git diff --check` затронутых файлов passed. [Кадры и логи](evidence/2026-10-01/bug-050/), [результат и ограничения](archive/2026-10-01/bug-050.md). T020 снят как code-fixed с проверенной native presentation.

## Bug 56 — /help и /redeem, 2026-10-01

- RustyFusion: `cargo test --lib chat_command_tests` — 6 passed; `cargo test --lib redeem` — окончательно 3 passed; `cargo test --lib test_player_save_reload` — 1 passed. **9 уникальных тестов** с учётом пересечения фильтров. Новая регрессия выдачи проверяет item type 7 / ID 119, слоты, SQLite reopen, repeat/stale claim, полный инвентарь, обмен, rollback при аварийном INSERT и успешный повтор после восстановления БД. Промежуточные падения подготовки fixture исправлены (runtime PC ID, свободный слот для аварийного теста, test DB error severity); окончательный запуск прошёл.
- `cargo check --lib --no-default-features --features postgres` и `cargo build --bin hybrid` — успешно. PostgreSQL БД не запускалась.
- `target/bug-056/localization-tests.exe --nocapture`: **1 passed**, текущий production `command_text.rs`, настоящий Localization и все 29 EN/RU ключей. Cargo client-bin тест и повторная собственная build-команда остановлены в общей очереди, не засчитаны.
- Полный клиент с `FFONE_PERF_OUTPUT=target/bug-056/ru|en`, `FFONE_PERF_CHAT_COMMANDS=1`, `--language ru|en`: **два exit 0**. RU — копия свежего Cargo binary; EN — успешная изолированная rustc-сборка текущего production main.rs и актуальной cached library. Enter, BUDDY → ALL и привязанные Bevy Text проверены. `chat-25/65-pass.txt`, четыре PNG и логи — `target/bug-056/{ru,en}`; RU/EN redeem PNG визуально осмотрены.
- UI — offline decoded-reply replay; серверная выдача — отдельный SQLite fixture. Реальных CodeItems в обоих текущих drops.json нет, TCP сценарий утверждённого промокода не заявляется. T042 снят как code-fixed: [Bug 56](archive/2026-10-01/bug-056.md).


## Bug 39 — 2026-10-01

RustyFusion: `cargo test --lib trade -- --test-threads=1` — 9 passed. Проверены handler offer/accept/refuse/cancel, forged/replayed acceptance, quantities, full-inventory swap, rollback, readiness reset и isolated SQLite save/reload. Клиент: свежий direct-rustc library/app/test build с существующими Cargo dependency artifacts; `target/bug039/client-tests.exe ui::pc2pc --skip default_asset_contract_is_safe_complete_and_source_exact --test-threads=1` — 25 passed; `characters::avatar_action` — 30 passed. Изменённый view с FocusPolicy::Pass и enabled controls также компилируется. Сохранённый asset hash test ранее failed: Taros actual 945A740F… vs expected 138BB…; он не объявлен passed. EN/RU JSON и адресный diff whitespace проверены.

Свежий полный exe 21:36 запускается, панель Trade From / Trade To осмотрена на RU с offline fixture PC 1/2, type 7 ID 90 qty 12, wallet 500. Native pointer/keyboard replay не завершён: computer-use дал дополнительный window 16×16 и не доставил события в клиент; повтор с WindowStyle Normal не помог. UI PASS-файл отсутствует, успешные interactions не заявляются. Login/shard 23000/23001 не слушаются, живых двух сессий нет. [Карточка и ограничения](archive/2026-10-01/bug-039.md).

## 2026-10-01 — Bug 52: красный фильтр предметов инвентаря

- `cargo test -p ffone-client --features diagnostics --lib inventory_ --locked`: **45 passed**, включая новую `inventory_availability_tracks_guide_level_and_base_identity` (guide/level/gender/base-vs-combined/Quest fallback). После отдельной компиляции того же lib-test source с зависимостями Cargo точный фильтр `inventory_availability --nocapture`: **1 passed, 2083 filtered out**; остальные тесты не запускались этим дополнительным прогоном.
- `cargo build -p ffone-client --bin ffone-client --features diagnostics --locked`: **PASS**. Общая очередь `target` задержала сборку и тесты. Финальный диагностический бинарник после корректировки проверочного сценария собран отдельно с аргументами rustc от Cargo, собственным output/incremental и копией готовой native UI library: **exit 0**. SHA256 проверенного `target/bug052-ffone-client.exe`: `C414102D371DC701F737620ACDC182D06153D092795754C89E8557B3803B8485`. Проверка компиляции `user_equip_ui_gpu_preview` с новым публичным контекстом в отдельном output: **PASS**.
- Полный клиент DX12, **1920×1080**, `FFONE_PERF_OUTPUT=target/bug052-inventory`, `FFONE_PERF_INVENTORY_AVAILABILITY=1`: **exit 0, 5/5 этапов PASS**. Weapon 2 (Ben, level 24), нейтральный weapon 10 (level 23), weapon 2 с внешностью 10: Dexter/36 → Ben/36 → Ben/23 → drag slot 0 → cancel. Проверены цвета, альфа, display и видимость корня. Все пять `inventory-stage-0..4.png` осмотрены: red → normal → red по ограничениям; drag сохраняет red с alpha .5, cancel возвращает alpha 1. Нейтральная иконка остаётся обычной, C-badge сохраняется.
- Артефакты: `target/bug052-inventory/`, `target/bug052-inventory-runtime.log`, `target/bug052-inventory-errors.log`. Первые два запуска остановились на проверочных сравнениях linear/Srgba white и округлении; они не засчитаны как PASS. Финальный запуск сравнивает ожидаемые значения sRGBA напрямую.
- Ограничение: реальный клиент/интерфейс/GPU с offline authoritative inventory и mentor/level; ручной диалог Guide Changer, серверный ответ и повторный вход не проверены. T021 снят как code-fixed: [карточка Bug 52](archive/2026-10-01/bug-052.md).
- `cargo build -p ffone-client --bin ffone-client --features diagnostics --locked`: **PASS**, завершённая сборка после исправления ошибок общего дерева.
- Первые два full-client запуска (`target/bug-043/ru/`, `ru-buttercup/`) остановились на выборе NPC с приветствием и квестовой строкой; успешной UI-приёмкой не считаются. Адресное чтение каталога подтвердило Larry 3000 / NPC 692, greeting и quest string 15083. Waypoint row 2876: native `(-4779.07, -52.61, 3477.06)`.
- Полный клиент от 21:22:18 Москва сохранён как `target/bug-043/ffone-client-natural.exe`. `FFONE_PERF_NPC_SPEECH=1`, `FFONE_PERF_NPCS=1`, `FFONE_PERF_POSITION="-4779.07 -52.61 3482.06"`, `--language ru` и `--language en`: **оба exit 0**, оба `npc-speech-pass.txt` — PASS, NPC 692 / строка 15083 отображается в течение четырёх кадров после запроса, до истечения приветствия. RU `greeting.png`/`quest.png` и EN `quest.png` осмотрены: новый пузырь виден над Larry 3000, старый текст исчезает, обе строки сохраняются в истории. Артефакты: `target/bug-043/ru-bot692/` и `target/bug-043/en-bot692/`.
- Живая проверка воспроизводит API публикации после offline-входа; серверный TaskStart/TaskEnd и десять минут фонового ожидания в живой сессии не выполнялись. Граница общего cooldown подтверждена регрессиями, не ускоренной записью реального сервера.

## 2026-10-01 — Bug 53: высота портретов Nano справа снизу

- Новый тестовый бинарник `target/debug/deps/ffone_client-300cdf94d89a9153.exe` от 21:05:54: `characters::gameplay_nano_portraits::tests::` — **7 passed**, `ui::gameplay::tests::layout::exact_retrobution_rectangles_are_stable` — **1 passed**. Новая регрессия проверяет сохранение экранных координат, видимость выше прежней границы, соответствие пропорций UI/render target и независимость journal.
- `cargo build -p ffone-client --bin ffone-client --features diagnostics --locked` — **PASS**. Полный offline-клиент: бинарники от 21:16:11 и 21:22:18, Южный Поки-Оукс, `FFONE_PERF_NANO_HUD=1`, freeze. A/B старого квадрата и новой области для Nano 24/42/43 и 6/12/35; каждый проход менял окно 1280×720 → 1024×768 → 1920×1080. **4/4 exit 0, 12 кадров**, assertions target sizes passed. Кадры исправленного режима осмотрены. Aku 35: возвращена небольшая срезанная часть верхнего контура рогов; масштаб/нижний край сохранены.
- [Шесть кадров высокой тройки до/после](evidence/2026-10-01/bug-053/). На 1024×768 раскрытый чат перекрывает счётчики в обоих режимах A/B; прежний отдельный layout-конфликт не исправлялся. На 1280×720 и 1920×1080 счётчики видны. Проверены шесть Nano/три разрешения, не весь каталог/все позы; серверные команды в offline replay не подтверждаются. FPS не заявляется.
- T033 снят как code-fixed; [карточка Bug 53](archive/2026-10-01/bug-053.md) архивирована с ограничениями.

### Bug 44 — полный клиент, завершено 2026-10-02

- `cargo build -p ffone-client --bin ffone-client --features diagnostics --locked`: успешно. Копия текущего exe в `target/bug044-runtime/ffone-client.exe`; `FFONE_PERF_OUTPUT=target/bug044-clear`, `FFONE_PERF_NPC_IDLE=1`, `FFONE_PERF_POSITION='-6434.3188 -56.4746 668.2039'`. Exit code **0**, stderr без panic/ERROR.
- NPC ID `1904461`, type 461 (Bad Max), Южный Пич-Оукс. Все **5** кадров `npc-idle-{90,180,270,400,520}.png` визуально осмотрены: модель целиком видна, разные idle-позы и native walk/run. `mob-animation-replay.json`: **112 наблюдений**, переходы idle при completions=1 предыдущего клипа, walk weight=1 / seek=0.6355 на 400, run weight=1 / seek=0.6833 на 520. Тестовые пакеты направлены в production ingress и рендер владельцев.
- Первые `target/bug044-idle` и `target/bug044-framed` не приняты для визуальной проверки (крупный план, затем листва); финальные кадры — `target/bug044-clear`. Менялась только opt-in камера fixture/положение запуска; GLB, принятые масштабы и IDs сохранены. Это проверка анимаций, не FPS-измерение.
- Живой shard и отдельный GPU-сценарий HNPC не проверялись; HNPC подтверждён адресной регрессией AnimationPlayer. Снято как code-fixed: [Bug 44](archive/2026-10-02/bug-044.md).

## 2026-10-02 — Bug 54 / T024: приёмка отображения

- cargo build -p ffone-client --bin ffone-client --locked — **success**; финальная повторная сборка после изменения fixture — **49.61 s**, success.
- Полный клиент, Sector V / NPC 2555, FFONE_PERF_NPC_BUBBLE_VISIBILITY=1: target/bug-054/client-building — **exit 0**, **11 assertions PASS**, 10 PNG. near/wall/wall-restored визуально осмотрены: показ перед зданием, скрытие за стеной, восстановление той же реплики. Collider variant 0932; координаты и bounds сохранены в wall.json.
- target/bug-054/client-stable (фонарный столб): **exit 0**, **11 assertions PASS**, все 10 PNG осмотрены; дальность и возврат подтверждены. client-wall (дерево): **exit 0**, 11 PASS, 5 кадров осмотрены; не использован как приёмка стены.
- Первый client-запуск timed out из-за несогласованных координат replay/capture и не засчитан; fixture исправлен. Проверка offline, координаты/камера заданы сценарием; renderer/UI/world geometry настоящие. Серверный проход и ручное управление не проверялись. [Результат Bug 54](archive/2026-10-02/bug-054.md).

## 2026-10-03 — Bug 76 / T124–T125

- `cargo check -p ffone-client --bin ffone-client` — success; существующие предупреждения в других модулях.
- Адресный `cargo test -p ffone-client --bin ffone-client user_equip_item_and_nano_modes_cancel_auto_run_and_release_input_on_close` ждал общий build lock и был остановлен без результата. Живой клиент и визуальная анимация не проверены. [Карточка](archive/2026-10-03/bug-076.md).

## Bug 25 — проверка по пользовательскому скриншоту

Пользовательское свидетельство сохранено в `evidence/bug-025-user-face-2026-10-01.png`: синие участки перекрывают нижнюю часть лица и область вокруг глаз. Свежий diagnostics example собран успешно. 12 парных male standup кадров yaw 45/65/85/275/295/315°, distance 1.4, обводка включена/отключена, и 12 кадров height/body 0/0, 2/1, 4/2 × pitch -25/-10/0/20°, yaw 315°, distance 2 — все exit 0, все осмотрены. Пятно не повторилось; production-шейдер не изменён, T056 активен. [Подробности и изображение](bug-025.md).

## 2026-10-03 — Bug 70 / T109

- `cargo test -p ffone-client --lib ui::gameplay::tests::npc_speech_priority -- --nocapture`: **3 passed**, 0 failed. Тип обычной реплики `Ordinary`, квестовой после прерывания очереди `Quest`.
- `cargo build -p ffone-client --bin ffone-client --features diagnostics --locked`: **success**. Бинарник сохранён в `target/bug-070/ffone-client.exe`.
- Полный клиент: `FFONE_PERF_NPC_SPEECH=1`, `FFONE_PERF_NPCS=1`, `FFONE_PERF_POSITION='-4779.07 -52.61 3482.06'`, `--language ru|en`: оба **exit 0**. Larry 3000 / NPC 692, Goat's Junkyard, квестовая строка 15083. Оба `npc-speech-pass.txt` подтвердили замену за четыре кадра. Снимки `target/bug-070/{ru,en}/greeting.png` и `quest.png` осмотрены: зелёное приветствие, жёлтая квестовая реплика, чёрный текст читается. Offline authority; серверное прохождение не проверялось. [Результат](archive/2026-10-03/bug-070.md).
