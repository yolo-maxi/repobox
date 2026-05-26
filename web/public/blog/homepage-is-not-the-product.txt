The Homepage Is Not the Product

2026-05-26 · product, positioning, websites

A small repo.box positioning lesson: the front page should explain the team, while specific products deserve their own pages.

Today we moved the repo.box Git permission-layer pitch off the homepage and into its own /git page.

That sounds like a tiny website edit. It was not. It was a product strategy decision wearing a CSS hoodie.


The mistake

We had let one product occupy the whole front door. repo.box started as a studio: an independent team building cool ideas with cool people. Then the Git layer got exciting. It had a crisp problem, a real demo, a command you could run, and a strong reason to exist. So naturally it expanded until the homepage looked like repo.box only meant one thing.

That is a common trap. When a product finally has shape, it produces more concrete copy than the umbrella brand. Concrete beats vague. “Git permission layer that makes repositories safe for agents” is easier to write than “we build useful weird things with taste.” So the product wins the page by sheer specificity.

But a homepage is not a trophy for whichever feature currently has the sharpest pitch.


The front door has a different job

A product page should convert. It can be narrow, opinionated, and full of implementation detail. It can say: here is the problem, here is the command, here are the rules, here is the demo.

A homepage should orient. It should answer a higher-level question: what kind of place did I just walk into?

For repo.box, the answer is not only “a Git security product.” That is one thing we are building. The broader answer is closer to: a small studio shipping experiments at the edge of agents, software, identity, and the web.

That distinction matters because the wrong front door quietly narrows the future. If the root domain becomes one product, every other project looks like a distraction. If the root domain communicates the studio, products can sit underneath it without fighting each other.


Umbrella first, product second

The fix was simple:

- Keep repo.box as the studio homepage.
- Move the Git permission-layer pitch to /git.
- Link to real pages only. No fake nav. No empty doors.
- Let each page do one job.

The Git layer did not get demoted. It got a better room. On /git, it can be specific without swallowing the whole identity. The install command, config example, playground, and permission model all belong there.

The homepage now has permission to be broader. It can point to projects, writing, the explorer, docs, and whatever we ship next. It can feel like a studio rather than a SaaS landing page that accidentally forgot pricing.


A useful rule

If your homepage can only describe one current feature, you probably do not have a homepage. You have a product page sitting at /.

That is fine for a single-product company. It is bad for a studio, a lab, a collective, or any team whose best work is still compounding across several directions.

The root should hold the identity. Subpages should hold the arguments.

This is especially important for small teams moving quickly. The newest project always feels like the whole universe because it is where the energy is. But the site should not be a mood ring. It should help visitors understand the shape of the work, not just the most recent obsession.


What we shipped

repo.box now has the split it should have had earlier:

- repo.box/ — the studio front page.
- repo.box/git — the Git permission-layer product page.
- repo.box/explore — the live repository explorer.
- repo.box/blog — the ongoing notebook.

Tiny change. Better architecture. Cleaner story.

The homepage is the lobby. Stop storing the whole product in the lobby.
