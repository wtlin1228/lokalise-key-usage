import { translate } from "@foo/i18n";

const LABELS = translate({
  title: "i18n.pet.party",
  desc: ["i18n.pet.party.desc", "lazy"],
  bird: {
    name: "i18n.bird",
    desc: ["i18n.bird.desc", "lazy"],
    size: {
      [SIZE.samll]: "i18n.bird.small",
      [SIZE.large]: ["i18n.bird.large", "lazy"],
    },
  },
  cat: {
    name: "i18n.cat",
    desc: ["i18n.cat.desc", "lazy"],
    size: {
      [SIZE.samll]: "i18n.cat.small",
      [SIZE.large]: ["i18n.cat.large", "lazy"],
    },
  },
  dog: {
    name: "i18n.dog",
    desc: ["i18n.dog.desc", "lazy"],
    size: {
      [SIZE.samll]: "i18n.dog.small",
      [SIZE.large]: ["i18n.dog.large", "lazy"],
    },
  },
});

const Card = () => {
  const b = LABELS.bird.name;
  const foo = foo.bar.abc;
  return (
    <div>
      <C>{LABELS.cat}</C>
      <D d={LABELS.dog[abc]}></D>
      {/* <E e={LABELS}></E> */}
    </div>
  );
};
