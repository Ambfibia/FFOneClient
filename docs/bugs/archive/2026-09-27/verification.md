# Проверки реестра — 2026-09-27

## Объект аудита

FFOneClient HEAD `3710295a5`, RustyFusion HEAD `faba1bc`, **с существующими незакоммиченными изменениями** обоих рабочих деревьев. Это не аудит одних только коммитов. Исходный список содержит 77 жалоб. Код/ассеты/настройки в этом аудите не изменялись.

## Выполненные тесты

Команды выполнялись из указанного репозитория. Все перечисленные ниже запуски завершились успешно; число — реально выполненные тесты, не filtered out.

### RustyFusion — 84 теста

| Команда | Passed | Связанные Bug |
| --- | ---: | --- |
| `cargo test --locked item_move_tests -- --test-threads=1` | 8 | 1 |
| `cargo test --locked --lib nano_acquisition_tests -- --test-threads=1` | 10 | 6 |
| `cargo test --locked --lib nano_attack_tests -- --test-threads=1` | 7 | 7 |
| `cargo test --locked --lib nano_tune_tests -- --test-threads=1` | 6 | 9 |
| `cargo test --locked --lib skills::damage_tests -- --test-threads=1` | 9 | 7 |
| `cargo test --locked --lib skills::cc_tests -- --test-threads=1` | 9 | 7 |
| `cargo test --locked --lib corruption::tests -- --test-threads=1` | 15 | 7 |
| `cargo test --locked --lib ai::tests -- --test-threads=1` | 10 | 19 |
| `cargo test --locked --lib chat_command_tests -- --test-threads=1` | 5 | 37 |
| `cargo test --locked --lib barber:: -- --test-threads=1` | 5 | 10 |

Предварительные фильтры `skills_damage_tests`, `skills_cc_tests`, `corruption_tests`, `ai_tests` дали ноль тестов из-за отличающихся имён модулей. Они не включены в результат; выше записаны повторные запуски с правильными фильтрами. Повторный запуск nano_tune_tests не удваивает число уникальных тестов.

### FFOneClient — 33 теста

| Команда | Passed | Связанные Bug |
| --- | ---: | --- |
| `cargo test -p ffone-client --lib objective_copy_tracks_right_menu --locked` | 1 | 36 |
| `cargo test -p ffone-client --lib selection_generation_tests --locked` | 1 | 27 |
| `cargo test -p ffone-client --lib ui::vendor::item_popup::tests --locked` | 13 | 18 |
| `cargo test -p ffone-client --lib ui::guide::tests --locked` | 17 | 36 |
| `cargo test -p ffone-client --lib short_content_keeps_frame --locked` | 1 | 16 |

Сборки выдали предупреждения существующего кода (unused imports/dead code/unreachable patterns и unnecessary braces); они не исправлялись в рамках документального аудита.

## Дополнительные проверки кода и данных

- Прочитаны текущие реализации обмена предметов, GM Nano, warp/vendor validation, поведения мобов, возврата к выбору, назначенных клавиш, race, соответствующие UI/audio diff и адресные регрессионные тесты.
- GLB JSON chunks прочитаны напрямую без изменения файлов: Mordecai/Titan имеют `nativeSurfaceStyle: cel` и отключённые outline passes; Van Kleiss не имеет cel-флага и имеет отключённый outline. Johnny Bravo содержит call/skill1/skill2/skill3; корректность самих движений по одним именам не установлена.
- Проверены `LegacyModelMaterial::enable_shadows() == false` и отдельный water fragment без fog stage.
- Проверены русский default и четыре заглавные tutorial.instruction строки в рабочем RU-bundle.
- Найден общий Nano sound-event runtime. Полнота событий/файлов для каждого Academy Nano не проверена.
- Старые отчёты gameplay/menu/visual corrections использованы как указатели; их GPU-результаты не считаются свежей приёмкой.

## Что не выполнено

Не запускались глобальная suite, live playtest, прослушивание звука, реальные gamepad interactions и свежие FPS/GPU-замеры. Не проверены версии уже работающих процессов. Сценарии с конкретными объектами/внешностью/размещениями остаются U или P, если в файле Bug нет более узкого подтверждения.

T означает исправление в проверенной логике, а не автоматическое закрытие всей интеграции. C означает конкретную реализацию без достаточной свежей приёмки. Исторический рассказ о ремонте нормалей Дарвина не подменяет текущий визуальный результат.

## Контроль документации

Все 77 SNN имеют ровно одного владельца Bug; 38 Bug имеют постоянные номера. Ссылки на локальные точки входа и Markdown-файлы проверяются перед сдачей. Реестр не создаёт новые задачи приложения и не запускает агентов.
