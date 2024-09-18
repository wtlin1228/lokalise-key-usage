import { translate } from "../utils/i18n";

const LABELS = translate({
  bird: "lokalise.key.bird",
  cat: "lokalise.key.cat",
  dog: "lokalise.key.dog",

  getBird: ["lokalise.key.get-bird", "lazy"],
  getCat: ["lokalise.key.get-cat", "lazy"],
  getDog: ["lokalise.key.get-dog", "lazy"],
});

const Foo = () => {
  return <div>{LABELS.bird}</div>;
};

const Bar = () => {
  return <div>{LABELS.getBird({})}</div>;
};
