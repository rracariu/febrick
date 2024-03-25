<script lang="ts">
	import * as Breadcrumb from '$lib/components/ui/breadcrumb/index.js';
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Card from '$lib/components/ui/card/index';
	import Input from '$lib/components/ui/input/input.svelte';
	import * as Tabs from '$lib/components/ui/tabs/index';

	import ScanSearch from 'lucide-svelte/icons/scan-search';

	import { onMount } from 'svelte';

	import brickTtl from '@brick';
	import { Brick } from 'febrick';

	let brick: Brick | undefined;

	const rootClass = 'Entity';
	let loadedClassNames: string[] = [];
	$: filteredClassNames = search.length
		? loadedClassNames.filter((entity) => entity.toLowerCase().startsWith(search.toLowerCase()))
		: loadedClassNames;

	let path: string[] = [rootClass];
	let search: string = '';

	onMount(() => {
		brick = new Brick(brickTtl);
		subClassOf(rootClass);

		console.log('Page mounted');
	});

	function subClassOf(name: string): void {
		loadedClassNames = brick?.subClassOf(name) ?? [];
	}

	function navigateToLevel(index: number): void {
		search = '';
		const part = path[index];
		path = path.slice(0, index + 1);
		subClassOf(part);
	}

	function navigateToClass(name: string): void {
		search = '';
		path = [...path, name];
		subClassOf(name);
	}
</script>

<Tabs.Content value="classes" class="space-y-4">
	<Card.Root class="col-span-4">
		<Card.Header>
			<Card.Title
				><div class="flex flex-row">
					<div class="flex-1">
						<Breadcrumb.Root>
							<Breadcrumb.List>
								<Breadcrumb.Separator />
								{#each path as part, index}
									<Breadcrumb.Item
										><Button
											class="text-primary p-0 text-current"
											variant="link"
											on:click={() => navigateToLevel(index)}>{part}</Button
										></Breadcrumb.Item
									>
									{#if index < path.length - 1}
										<Breadcrumb.Separator />
									{/if}
								{/each}
							</Breadcrumb.List>
						</Breadcrumb.Root>
					</div>
					<div class="flex flex-row gap-2">
						<Input placeholder="Find class..." bind:value={search} />
					</div>
				</div></Card.Title
			>
		</Card.Header>
		<Card.Content>
			<div class="grid grid-cols-4">
				{#if brick}
					{#each filteredClassNames as className}
						{@const desc = brick?.classDescription(className)}
						<Card.Root class="m-1"
							><Card.Header>
								<Card.Title>
									<Button
										class="text-primary m-0 gap-1 p-0 text-current"
										variant="link"
										on:click={() => {
											navigateToClass(className);
										}}><ScanSearch /> {className}</Button
									>
								</Card.Title>

								<Card.Description class="overflow-hidden truncate"
									>{desc.definition}</Card.Description
								>
							</Card.Header>
						</Card.Root>
					{/each}
				{/if}
			</div>
		</Card.Content>
	</Card.Root>
</Tabs.Content>
