import { addMessages, init } from "svelte-i18n";
import ptBR from "../lib/i18n/pt-BR.json";
import enUS from "../lib/i18n/en-US.json";

addMessages("pt-BR", ptBR);
addMessages("en-US", enUS);

init({
  fallbackLocale: "pt-BR",
  initialLocale: "pt-BR",
});
