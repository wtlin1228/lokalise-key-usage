import { translate } from "../utils/i18n";

const LABELS = translate({
  a: {
    b: {
      bird: "lokalise.key.bird",
    },
    c: {
      cat: "lokalise.key.cat",
    },
  },
  b: {
    b: {
      getBird: ["lokalise.key.get-bird", "lazy"],
    },
    c: {
      getCat: ["lokalise.key.get-cat", "lazy"],
    },
  },
  dog: "lokalise.key.dog",
  getDog: ["lokalise.key.get-dog", "lazy"],
});

const Foo = () => {
  return <Comp labels={LABELS.a} />;
};

const Bar = () => {
  return <Comp labels={LABELS.b} />;
};

const Comp = ({ labels }: { labels: any }) => {
  return <div>{labels}</div>;
};
