# System messages

Keep the source manager/element ownership; there is no inferred `Cn...` replacement
class. Dialog=600×164; content=(140,20,420,104); OK=(454,124,110,25),
Cancel=(44,124,150,25), ExitCreation=(320,124,245,25). Dimmer alpha=.75;
border=(40,16,40,40). LIFO capacities are 10/10; newest message alone owns controls.

Renderable types: 0,1,2,3,6,7,8,9,12,13,14,15. Normalize 10/11 to 1/2; invalid
4/5 fail rather than guess a layout. Decorative label children pass focus through to
buttons. Preserve typed owner callbacks, disabled-state gating and modal input order;
dismissal must not leak a same-frame action to the underlying service.
