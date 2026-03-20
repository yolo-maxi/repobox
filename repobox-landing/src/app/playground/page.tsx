import { Metadata } from "next";
import { PlaygroundClient } from "@/components/playground/PlaygroundClient";
import { RegMarks } from "@/components/RegMarks";

export const metadata: Metadata = {
  title: "repo.box — Playground",
  description:
    "Try repo.box config generation. Describe permissions in English or paste a .repobox.yml to understand it.",
};

export default function PlaygroundPage() {
  return (
    <>
      <RegMarks />
      <PlaygroundClient />
    </>
  );
}
