<script lang="ts">
	import type { BrickProperty, Curie } from 'febrick';

	import { Button } from '$lib/components/ui/button/index.js';
	import Badge from '$lib/components/ui/badge/badge.svelte';
	import * as Table from '$lib/components/ui/table/index.js';

	let { property, navigate }: { property: BrickProperty; navigate: (curie: Curie) => void } =
		$props();

	function constraints(prop: BrickProperty): { title: string; curies: Curie[] } | undefined {
		if (prop.logicalConstraints.length) {
			const constraint = prop.logicalConstraints[0];
			if ('Or' in constraint) {
				return { title: 'Any of', curies: constraint.Or.map((prop) => prop.class) };
			} else if ('And' in constraint) {
				return { title: 'All of', curies: constraint.And.map((prop) => prop.class) };
			} else if ('Not' in constraint) {
				return { title: 'None of', curies: constraint.Not.map((prop) => prop.class) };
			} else if ('XOne' in constraint) {
				return { title: 'One of', curies: constraint.XOne.map((prop) => prop.class) };
			} else {
				return undefined;
			}
		} else {
			return undefined;
		}
	}

	function curieToString(c: Curie | null | undefined): string | undefined {
		return c ? c.prefix + ':' + c.localName : undefined;
	}
</script>

<Table.Row>
	<Table.Cell class="justify-self-end border-b-2 border-b-slate-50 align-text-top">
		{property.path}
	</Table.Cell>
	<Table.Cell class="justify-self-start border-b-2 border-b-slate-50 align-text-top">
		{#if property.class.localName}
			<Button variant="link" onclick={() => navigate(property.class)}
				>{property.class.prefix}:{property.class.localName}</Button
			>
		{:else}
			{@const dt = curieToString(property.datatype)}
			{@const nodeKind = curieToString(property.nodeKind)}
			{@const { title, curies } = constraints(property) ?? {}}
			{#each curies ?? [] as c, i}
				<div class="m-1 flex content-center items-center">
					{#if i === 0}
						<Badge variant="outline" class="mr-1">{title}:</Badge>
					{/if}
					<Button variant="link" onclick={() => navigate(c)}>{c.prefix}:{c.localName},</Button>
				</div>
			{/each}
			{#if !curies || curies.length === 0}
				{#if dt}
					<Badge variant="outline">{dt}</Badge>
				{/if}
				{#if nodeKind}
					<Badge variant="outline" class="ml-1">{nodeKind}</Badge>
				{/if}
			{/if}
		{/if}
	</Table.Cell>
</Table.Row>
