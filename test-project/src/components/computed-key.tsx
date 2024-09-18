import { translate } from "../utils/i18n";

const LABELS = translate({
  a: {
    b: {
      c: {
        cat: "lokalise.key.cat",
        dog: "lokalise.key.dog",
        bird: "lokalise.key.bird",
      },
      d: {
        getBird: ["lokalise.key.get-bird", "lazy"],
        getCat: ["lokalise.key.get-cat", "lazy"],
        getDog: ["lokalise.key.get-dog", "lazy"],
      },
    },
  },
});

const Foo = (type: "cat" | "bird") => {
  return <div>{LABELS.a.b.c[type]}</div>;
};

const Bar = (type: "cat" | "bird") => {
  return <div>{LABELS.a.b.d[type]({})}</div>;
};
