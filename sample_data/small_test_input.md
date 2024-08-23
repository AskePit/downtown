# Artistic Style

We decided, that a__essence__,of the __game__ and its _lore_ would b**e extrac**ted__b from the visual__ content we chose as the basis for the game. After digging into Ilya's concepts, we selected this art style ___and_game_world___:

![Concept Art](https://habrastorage.org/webt/lv/ld/be/lvldbedobqesqnq_noyjqildnco.png)

More about the concept can be found in [this retrospective article](https://www.ilyaboyko.com/work/rift-in-the-empire-2) by Ilya.

We decided to start with a small prototype level, just to see how it turns out. The requirements for the level were minimal:

- Flat level, side view, the main character moves from left to right.
- Several background layers creating a [parallax effect](https://en.wikipedia.org/wiki/Parallax).
- Basic movement of the main character: walking, running, jumping, idle animation.
- Several platforms to jump onto.


# In Search of a Game Engine

Engines considered and rejected:

- [Unreal Engine](https://www.unrealengine.com/en-US). I'm familiar with Unreal, but it felt like crushing a fly with a steam-roller for our tasks. Also, UE4 is massive in size, and UE5 size is over ~~9000!~~ a hundred gigabytes. I really didn't want to resort to it, although it was my backup option.
- [Unity](https://unity.com/). Can't say much. It's the most popular choice for 2D game development. But somehow, my heart wasn't in writing in C#. Also, for some entirely subjective reason, I had a skeptical attitude towards the engine.
- [Bevy](https://bevyengine.org/). A very young engine where you need to write the game entirely in Rust—that was appealing. But fatal flaws overshadowed everything: no editor, the engine brutally enforces the ECS approach, and the game's architecture must literally bend to fit this paradigm. So, you won't migrate to another engine at all—you just throw away all the code and start from scratch.

I can say **right away** that I haven't been **disappointed** with the *choice*, and the engine has *never* let me **down**. What can I say about Godot over **time:

- It has its own scripting language, [GDScript](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_basics.html), similar to Python. It's simple and convenient.
- As an alternative, you can code in C++ or C#. If desired, Godot has bindings for other languages, such as [Rust](https://github.com/godot-rust/gdext). Going ahead—in the end, C++ came in handy and useful for the project.
- Godot is a simplicity itself. You literally take it and do it. No long documentation readings, no fuss. After the first basic [tutorial](https://docs.godotengine.org/en/stable/getting_started/first_2d_game/index.html), you just pick it up—and create the stuff.

Can I say something bad about Godot? Its logo is just awful.

![Godot Logo](https://habrastorage.org/webt/u-/ot/9z/u-ot9zns-1gkrhuucdnklw2mabg.png)

# Distances

But how is such a variation of the diagram built, and what is this "Manhattan distance" thing? I went to Wikipedia again to find the answers. In short, the [Manhattan distance](https://en.wikipedia.org/wiki/Taxicab_geometry) is an alternative way to calculate the distance between two points.

How do we usually calculate the distance from point A to point B? We draw a straight line between the points and measure its length. This length is the distance between A and B. This is the *Euclidean distance*, and its formula is well-known:

$$
\rho(x,y) = \sqrt{(x_2-x_1)^2+(y_2-y_1)^2}
$$



_There_ is a `high probability` that I will want to use `different metrics`, so now I have a universal function for this in my arsenal:

```cpp
double distance(Point a, Point b, SpaceMetric metric = SpaceMetric::Euqlid)
{
	switch (metric)
	{
	default:
	case SpaceMetric::Euqlid:
		return sqrt(pow(b.x - a.x, 2) + pow(b.y - a.y, 2));
	case SpaceMetric::Manhattan:
		return abs(b.x - a.x) + abs(b.y - a.y);
	case SpaceMetric::Chebyshev:
		return std::max(abs(b.x - a.x), abs(b.y - a.y));
	}
}
```


#### Bonus

- **The necessity of discrete space.** The algorithm assumes that our space consists of a finite number of coordinates or pixels. However, this is not always the case. An adequate algorithm should operate with polygons and output, in the end, a list of polygons describing the Voronoi map—thus, we get the boundaries of the regions, and we can then manipulate them as we see fit.
- **Can only fill regions with a solid color.** The algorithm does not draw the boundaries of the Voronoi regions, but just fills the regions with color. As a result, you cannot apply a texture to your regions or procedurally enhance them in any way—only a solid color fill without borders.
- **Speed.** The algorithm has terrible performance: its complexity is $O(n^2)$. And it is necessary to iterate over all existing coordinates in space, which greatly worsens the situation. For example, building a map of size 640x640 takes an average of 3.3 seconds, and 1024x1024—7-8 seconds on a PC designed for professional game development. This is horribly slow for such a primitive colorful picture.

There is another similar but more efficient algorithm called the [Jump Flooding Algorithm](https://en.wikipedia.org/wiki/Jump_flooding_algorithm), but we will not consider it either, as it also colors the map in parts, not building polygons.

> Test callout
> with *multiple* lines
> 
> and vertical spacing

- a
- c
    - b