# Проверки актуальной очереди

## 2026-10-05 — актуализация реестра

Сверены 79 непустых ячеек нового текста, прежние открытые карточки и записанные результаты исправлений. Архив и старые реестры удаляются по запросу пользователя; проверенные раньше результаты, относящиеся к открытому остатку, сохранены в карточках. Текущие сценарии воспроизводятся при исполнении конкретного Bug N.

Runtime-тесты, сборка и живой клиент в этой актуализации не запускались. Прежние тесты не объявляются выполненными повторно.

Выполнены адресные проверки документов:

- 38 карточек совпадают с 38 задачами в README; удалённые Bug 62/67 объединены в Bug 13/26, номера не переиспользованы.
- Все 79 новых пунктов T169–T247 имеют ровно один статус: 52 актуальных остатка, 27 исключений с основаниями. Всего 134 уникальных T-якоря с сохранёнными источниками прежних открытых задач.
- Все локальные Markdown-ссылки и ссылки на T-якоря в документах очереди разрешаются; ссылок на удалённый архив/старый реестр нет. Каждый документ корня очереди меньше 1 000 строк.
- В archive осталось 0 файлов: удалены 98 архивных записей. Удалены старый реестр 2026-10-02, две объединённые карточки и две устаревшие TSV-таблицы. Новый исходный TSV сохранён побайтно из вложения пользователя.
- `git -c core.safecrlf=false diff --check -- AGENTS.md docs/bugs` — passed.

## 2026-10-08 — Bug 19: скорость мирных маршрутов

Установлены конкретные объекты: Numbuh 2020 (type 2895, placement 136, route 30),
ghostduck (type 2596, placements 936–938, routes 100/99/98), дорожные машины
(types 2917–2921, 47 привязанных маршрутов). Скорости маршрутов 300/300/900
превышали XDT Walk 200/100/800, хотя сервер уже выбирал `iMoveStyle=Walk`.
RustyFusion ограничивает фактическую скорость мирного authored route табличной
скоростью ходьбы; точки и исходные скорости маршрута не изменяются. Боевые пути,
отступление, эскорты, более медленные участки и маршруты без Walk speed сохраняют
свои скорости.

Выполнено:

- `cargo test --lib bug019_` в RustyFusion — 2 passed: bundled girl/duck/car,
  скорость пакета и серверного перемещения, повторный выход/вход наблюдателя,
  сохранение исходного маршрута, медленный участок, combat/retreat paths.
- `cargo test --lib authored_npc_route_survives_idle_ai_and_reaches_both_watchers`
  — 1 passed; authored patrol моба сохраняет прежние скорость и Walk.
- `cargo test --lib escort` — 5 passed.
- `cargo test --lib placement_paths_follow_source_ids_across_channels_and_honor_loop_flag`
  — 1 passed.
- `cargo test -p ffone-client --lib npc_motion_` — 2 passed: горизонтальное
  движение и непрерывный Walk между сегментами входящего потока.
- Сборки `hybrid-bug019-final` и клиента с opt-in probe — passed.

В живом клиенте обнаружена вторая причина: оба общих skeleton GLB не содержали
`walk`, поэтому HNPC переходил на `stand1` при корректном серверном Walk.
В FusionForge расширен существующий прямой адаптер: `convert-player-walk`
восстанавливает точные клипы #34550 (male) и #34461 (female) из primary
CharacterSelection.resourceFile, проверяя Animation owner и digest контейнера.
Добавлен native-каталог locomotion_animations.json и его потребитель в клиенте.

- Read-only preflight без разрешения замены отклонил конфликт существующих GLB;
  preflight с `--replace-existing` и прямое добавление — passed.
- Сравнение GLB до/после: прежние animations/accessors/bufferViews, прочие JSON
  структуры и исходный BIN prefix сохранены; добавлен ровно один `walk` на пол.
- Повторные прямые импорты — passed; SHA256 всех трёх выходов совпадают побайтно.
- Неподдерживаемый источник отклонён до записи; SHA256 всех трёх выходов сохранены.
- `cargo test -p ffone-asset-pipeline --lib model_animation_append` в FusionForge
  — 4 passed: запрет замены клипов, сохранение исходного GLB и проверка кривых
  при повторном добавлении после переноса буферов.
- Промежуточный запуск настоящего клиента: girl NPC 403, speed 2.0, style 0,
  HNPC applied `walk`, repeat=true; снимок подтверждает видимую ходьбу.
- `cargo test -p ffone-client --lib bug019_civilian_walk_is_available_on_both_shared_rigs`
  — 1 passed: оба GLB содержат циклический Walk с индексом и числом каналов из
  native-каталога; Run и Stand1 доступны вместе с ним.
- В собранном test executable выполнен
  `characters::player_shared_rig::tests::production_catalog_verifies_both_native_shared_rigs --exact`
  — 1 passed: общий production-каталог и принятые части обоих ригов.

Полный прогон трёх типов с повторным streaming ожидает свежей сборки probe.
Тестовая SQLite, локальные порты 23919/23920, логи и снимки находятся в ignored
`target/bug019`; тестовые персонажи и перемещения выполнялись в отдельной SQLite,
сохранение пользовательских настроек клиента отключено.

## 2026-10-08 — Bug 8: исходный порядок зиплайна и пушки

По явному запросу пользователя применён reverse-engineering skill FusionForge.
Read-only IL-разбор ограничен `cnAvatarThirdPersonMove` (StartZipline,
StartLauncher, EpUpdate, Jump), `cnAvatarAnimation` (AvatarZipline,
AvatarLauncher, Update), `cnAvatarStatus::LoadZiplineObject` и `cnLauncher`
(ReceiveLuncherInit, CameraUpdate, Update). Primary: Retrobution20260821,
`main.unity3d`, SHA256
`01b544976b2d54355507cf30fe6dfada2b476b92b209a3d47c1499669ed9b4ef`.
Контрольные объекты map_00_09: launcher cneId 278/objectId 402000003/node #5447,
zipline cneId 277/objectId 402000002/node #5442; скорость троса 6 native units/s.
Unity runtime, bundles и conversion tooling в FFOneClient не добавлялись.

Восстановлены CCT Move к точке троса с исходным bias, последующее вычитание
hand-to-toe height и пакет из итоговой позиции. Оба выхода с троса и боковое
столкновение после выстрела переходят через Jump(0), очищают старое sliding
состояние и блокируют повторный прыжок в воздухе. Первый LAUNCHER содержит
полный вектор; продолжения — горизонтальный; завершающий пакет нисходящего
бокового столкновения сохраняет отрицательную вертикальную скорость.
Скрытые при прицеливании renderer roots защищены от повторного включения
системой дальности. Исходник скрывает пушку/аватара и вращает камеру; отдельный
снаряд или вспышка в проверенной ветке выстрела не создаются.

Выполнены адресные проверки, глобальные suites не запускались:

- `cargo test -p ffone-client --lib world_systems::behaviour::tests::traversal --locked`
  — 6 passed: первый/последующие пакеты, floor/side завершение, прыжок и конец
  троса, позиция после CCT до hang offset.
- Свежий lib test executable: `gameplay::movement::` — 41 passed;
  `characters::avatar_action::` — 31 passed; `ui::launcher::` — 13 passed;
  `tutorial_runtime::player_rig_runtime::` — 31 passed. Всего 122 адресных теста.
- Базовый opt-in прогон настоящего клиента до последних изменений — passed;
  `target/bug-008-reverse/baseline`: 12 кадров и traversal.json. Кадр 320
  подтверждает видимый trolley, ropedown и скрытое оружие. Это offline fixture,
  не проверка обмена с сервером или удалённого игрока.

Первая сборка exe выявила девять ошибочных `crate::tutorial_mission_content`
ссылок в текущих правках маркеров миссий. Исправлен только путь к модулю
библиотеки (`ffone_client::tutorial_mission_content`), без изменения их логики.
