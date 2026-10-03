# Исторический реестр разрывов

Срез из прежнего backlog (сентябрь 2026), не текущая очередь задач. Некоторые пункты
ниже описаны до последующих исправлений. Перед работой проверить только выбранный
пункт по коду, тесту и актуальному контракту экрана; не запускать весь реестр и не
переобъявлять реализованное отсутствующим. ID сохранены для старых ссылок.

Повторные дневники публикаций удалены. Решения по ID/донорам принадлежат production
таблицам и FusionForge `docs/source-contract.md`, не старым числам переписи.
Критерии проверки: [parity](parity.md). Реестр жалоб владельца и его датированные статусы:
[FusionForge completion plan](../../FusionForge/docs/reference/evidence/cases/ffone-completion-plan-20260905.md).


Приоритет: **P0** — блокирует заявку «играбельный клиент», **P1** — блокирует
заявку 1:1 по разделу, **P2** — полнота.

## EPIC A — Устойчивость сессии (P0)

- **A1.** Реконнект, таймаут, дубликат логина, разрыв соединения.
  DoD: сессия дольше 30 минут без предупреждений последовательности;
  принудительный разрыв восстанавливается без расхождения состояния персонажа
  с сервером.
- **A2.** Cookie/session-авторизация и все ветки отказа логина
  (коды `LS2CL_REP_LOGIN_FAIL`). DoD: каждая ветка отказа покрыта тестом.
- **A3.** Недостающие пакеты логин-сервера: `SHARD_LIST_INFO`, `VERSION_CHECK`,
  `CHECK_NAME_LIST`, `SERVER_SELECT`, `CHAR_SELECT_SUCC/FAIL`.
  DoD: клиент работает и с OpenFusion, и с ABI оригинального LS.
- **A4.** Запись и воспроизведение пакетов с вычисткой секретов.
  DoD: golden-реплей журнала логин→мир проходит в CI.
- **A5.** Ограниченный транспорт worker→Bevy с backpressure.
  DoD: документированный бюджет очереди, тест на переполнение.

## EPIC B — Бой (P0)

- **B1.** `PC_ATTACK_CHARS`, `NPC_ATTACK_CHARS`, `CHARACTER_ATTACK_CHARACTERS`.
  Источник: `cnAvatarAttack`, `sCAttackResult`.
- **B2.** Reply-хвосты ракеты и гранаты: `*_STYLE_READY`, `*_STYLE_HIT`,
  `*_STYLE_FIRE_SUCC`, `NPC_BULLET_STYLE_HIT`, `NPC_GRENADE/ROCKET_STYLE_FIRE`.
- **B3.** Симуляция снарядов: `BulletMoveScript`, `BulletContainer`,
  `BulletGenData`, `BulletRPSData`, `sBulletAppearanceData`, `BulletTable`.
  DoD: траектория, время жизни и точка попадания совпадают с оригиналом
  в пределах объявленного допуска.
- **B4.** Реакции на урон и смерть: `DamageMotion`, `DeadMotion`, `AppearEffect`,
  `FloatingPoint` (всплывающие числа), `MegaAttackMark`, `SwordTrailController`.
- **B5.** Состояние боя: подключить `COMBAT_BEGIN`/`COMBAT_END` к продакшену,
  боевая рамка по `game-frame-combat-primary-evidence.md`.
- **B6.** `PC_SKILL_USE`, `SKILL_ADD`, `SKILL_DEL` — скиллы предметов и гумболы,
  потребитель `m_pSkillTable` и `m_pSkillBookTable`.
- **B7.** Перегрев оружия по живой экипировке — `family.overheat` сейчас
  принудительно скрыт.

## EPIC C — Экономика и инвентарь (P1)

- **C1.** Bank: подключить `ffone-net` → `ffone-client-network` → `app`,
  маршрут NpcIcon, drag&drop, delete и disassemble.
- **C2.** Trade pc2pc: весь ABI (30 пакетов), приглашение через NpcIcon,
  транзакционное применение итога. DoD: два локальных клиента проходят
  полный цикл, включая отмену и разрыв связи.
- **C3.** Email: 23 пакета, вложения, почтовые расходы, приём одиночный и всех.
- **C4.** UserStore/StreetStall: 22 пакета, три режима (`MyStore`,
  `UserStore`, `ReturnStore`), регистрация цен.
- **C5.** Enchant и Combi: `ITEM_ENCHANT`, `ITEM_COMBINATION`, `GIVE_ITEM`.
- **C6.** Общий рендер попапов `EquipPopup`/`GumPopup`/`TuringPopup` и захват
  мыши для drag&drop — сейчас блокирует Vendor, Bank и QuickSlot одновременно.
- **C7.** Реальные NPC-камеры (render target 200×150) для Vendor, Trade, Race.

## EPIC D — Мир и перемещение (P1)

- **D1.** Инстансы: `INSTANCE_MAP_INFO`, `m_pInstanceData` (152), Infected Zones.
- **D2.** Варпы: `MAP_WARP`, `TRANSPORT_WARP`, `WARP_CHANNEL`,
  `REGIST_TRANSPORTATION_LOCATION`, `m_pWarpData` (321).
- **D3.** Транспорт: `cnBusMoveController`, `cnBusContainer`, `TransportCache`,
  `TransportMethod`, `m_pTransportationData` (115).
- **D4.** Shiny/яйца: `SHINY_PICKUP_SUCC/FAIL`, `cnShinyController`,
  `m_pShinyData` (130).
- **D5.** `cnWarHead` и `cnWarHeadContainer`.
- **D6.** Ездовые (`PC_RIDING`), `PC_BROOMSTICK_MOVE`, `PC_BELT`, `PC_ROPE`.
- **D7.** Каналы шарда: `PC_CHANNEL_NUM`, `CHANNEL_INFO`, `WARP_CHANNEL`.
- **D8. Снято.** `map_00_08`, `map_01_09` и `map_11_15` публиковать нечего: это
  позиции-заглушки, чей `DongResources_*` содержит только террейн тайла 00_09.
  Реестр из 170 тайлов полон. Доказательства — в FusionForge `docs/legacy/ffone/world-static-extraction.md`.
  Старый многоэтапный world-пайплайн — долг инструмента, а не повод создавать
  промежуточные файлы. Новые тайлы должен обслуживать прямой конвертер.
- **D9.** `DynamicEntityCulling` и `UserWaypoints`.

## EPIC E — Социальное и чат (P1)

- **E1.** MenuChat целиком: `m_pChatTable` (1017 фраз, 105 вторичных,
  11 третичных), `m_pMenuChatEmoteData`.
- **E2.** Чат-пузыри над персонажами (`eChatbubbleType`).
- **E3.** Эмоции и макросы: `m_pEmoteTable` (35 анимаций, 17 текстур).
- **E4.** `SlangChecker` и `m_pFilterTable`.
- **E5.** Блок-листы: `SET_PC_BLOCK`, `SET_BUDDY_BLOCK`.
- **E6.** Buddy: `BUDDY_WARP` (в том числе другой шард), `GET_BUDDY_LOCATION`,
  `GET_BUDDY_STYLE`, `GET_GROUP_STYLE`, `GET_MEMBER_STYLE`.
- **E7.** Меню игрока (NpcIcon) для приглашения в группу и `LEAVE GROUP`.
- **E8.** `ANNOUNCE_MSG` и `FE2CL_ERROR`.

## EPIC F — Миссии (P1)

- **F1.** Полный каталог Guide/Nano/Normal вне туториала:
  `m_pMissionData` 2866, `m_pJournalData` 2991, `m_pRewardData` 723.
- **F2.** `MISSION_COMPLETE`, `TASK_CONTINUE`, `KILL_QUEST_NPCS`.
- **F3.** Варианты задач: repeat, escort, checker
  (`cnMissionTaskCheckerNode`, `cnNPCAndMissionRelationNode`).
- **F4.** Отказы abandon и delete, общий мировой lifecycle журнала.

## EPIC G — Режимы без владельца (P2)

- **G1.** `cnHelpMode` / `cnGuiHelp` и `m_pHelpTable`.
- **G2.** `BarberMode` / `BarberGui`, `PC_BARBER_OPEN` и `PC_BARBER_CONFIRM`.
- **G3.** Катсцены: `cnMoviePlayer`, `AutoplayMovie`, `m_pCutSceneTable`.
- **G4.** Сезонные события: `GameEventManager`, `HalloweenEvent`, `KnishmasEvent`,
  `PC_EVENT`, `sUpdateEventCondition`.
- **G5.** `cnFirstUseSysManager` и `m_pFirstUseTable`.
- **G6.** EP-гонки: подключить готовую машину состояний
  (`race_ui/mode.rs`, `race_ui/rank.rs`) к 18 пакетам и к живым кольцам
  и чекпоинтам.
- **G7.** `NetworkStats` и диагностический оверлей.

## EPIC H — Контент (P2)

Точная перепись: `../FusionForge/tools/legacy-sources/asset-census.py`.

- **H1.** Опубликовать 19 моделей персонажей, которые есть в чистом билде.
  Экспорт и публикация работают; блокер — обязательное GPU-свидетельство
  `ffone.logical-model-gpu-evidence.v1` на каждую модель.
- **H2. Сделано.** Четыре модели оружия установлены; экипировка закрыта, публикуемых
  пропусков не осталось. Эффекты выстрелов у трёх из четырёх разрешаются; у Wilt's
  Basketball Blaster — нет, и это дефект данных оригинала, а не пробел публикации.
- **H2б.** Отдельным решением владельца: 7 моделей персонажей, существующих только
  в донорском билде Academy. Это расширение, не паритет.
- **H3.** Русская озвучка: 1059 → 5311 файлов, либо явно зафиксировать
  политику фолбэка на EN и покрыть её тестом.
- **H4.** Проверить потребителей конкретных неиспользуемых XDT-секций по статическому аудиту FusionForge; лексическое отсутствие не доказывает пропуск.

## EPIC I — Доказательная база (P0, параллельно всему)

- **I1.** Актуализировать `b8c3-native-port-coverage.json`: сейчас 9 записей
  на 264 класса оригинала. Довести до полного реестра со статусами.
- **I2.** Заморозить эталоны: пакетные потоки оригинала и FFOne для выбранных
  маршрутов. Историческая приёмка не означает проверку нынешней версии; см. `parity.md`.
- **I3.** Нормализованные сравнения 1264×681 «оригинал против нативного» для
  всех 41 записи parity-matrix. Сейчас почти везде есть только acceptance-кадр
  FFOne, а не сравнение с оригиналом.
